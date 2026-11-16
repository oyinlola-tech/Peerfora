#[path = "moderation_reports_postgres/support.rs"]
mod support;

use axum::http::StatusCode;
use peerfora_api::{app::build_app, database::connect_database, utils::one_time_token};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{
    TEST_ORIGIN, create_account_fixture, create_community_post, create_public_community,
    response_json, send_json_request, test_app_state,
};
use uuid::Uuid;

#[tokio::test]
async fn reports_are_target_scoped_permission_gated_and_audited() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let author = create_account_fixture(&database, "report_author").await;
    let reporter = create_account_fixture(&database, "report_user").await;
    let app = build_app(test_app_state(database.clone(), database_url.clone()));
    let community_slug = create_public_community(&app, &author.session_token).await;
    let post_id = create_community_post(&app, &community_slug, &author.session_token).await;

    let report_response = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/posts/{post_id}/reports"),
        &reporter.session_token,
        json!({"category":"spam","description":"This post advertises an unrelated service."}),
    )
    .await;
    assert_eq!(report_response.status(), StatusCode::CREATED);
    let report = response_json(report_response).await;
    assert_eq!(report["status"], "open");
    let report_id = report["id"].as_str().expect("report includes an ID");

    let duplicate_report = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/posts/{post_id}/reports"),
        &reporter.session_token,
        json!({"category":"spam","description":"Repeated active report."}),
    )
    .await;
    assert_eq!(duplicate_report.status(), StatusCode::CONFLICT);

    let (administrator_id, administrator_token) =
        create_administrator_session(&database, "trust_safety_lead").await;
    let student_cannot_review_reports = send_json_request(
        &app,
        "GET",
        "/api/v1/admin/reports",
        &reporter.session_token,
        json!({}),
    )
    .await;
    assert_eq!(
        student_cannot_review_reports.status(),
        StatusCode::UNAUTHORIZED
    );

    let report_list = send_admin_request(
        &app,
        "GET",
        "/api/v1/admin/reports?status=open",
        &administrator_token,
        None,
    )
    .await;
    assert_eq!(report_list.status(), StatusCode::OK);
    let report_list: Value = response_json(report_list).await;
    assert_eq!(report_list["totalCount"], 1);
    assert_eq!(report_list["items"][0]["targetType"], "post");

    let assigned = send_admin_request(
        &app,
        "POST",
        &format!("/api/v1/admin/reports/{report_id}/assign"),
        &administrator_token,
        Some(json!({"administratorId":administrator_id})),
    )
    .await;
    assert_eq!(assigned.status(), StatusCode::NO_CONTENT);
    let assignment_audit_reason: String = sqlx::query_scalar(
        "SELECT reason FROM administrative_audit_logs
        WHERE actor_administrator_id = $1 AND target_id = $2
            AND action = 'moderation.report.assigned'",
    )
    .bind(administrator_id)
    .bind(report_id.parse::<Uuid>().expect("report ID is a UUID"))
    .fetch_one(&database)
    .await
    .expect("report assignment audit entry should exist");
    assert_eq!(
        assignment_audit_reason,
        format!("assigned to administrator {administrator_id}")
    );

    let (_, support_agent_token) = create_administrator_session(&database, "support_agent").await;
    let support_agent_cannot_resolve = send_admin_request(
        &app,
        "POST",
        &format!("/api/v1/admin/reports/{report_id}/resolve"),
        &support_agent_token,
        Some(json!({"decision":"dismiss","reason":"Support lacks this permission."})),
    )
    .await;
    assert_eq!(support_agent_cannot_resolve.status(), StatusCode::FORBIDDEN);

    let resolved = send_admin_request(
        &app,
        "POST",
        &format!("/api/v1/admin/reports/{report_id}/resolve"),
        &administrator_token,
        Some(json!({"decision":"dismiss","reason":"Review found no policy violation."})),
    )
    .await;
    assert_eq!(resolved.status(), StatusCode::NO_CONTENT);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrative_audit_logs
        WHERE actor_administrator_id = $1 AND target_id = $2
            AND action = 'moderation.report.dismissed'",
    )
    .bind(administrator_id)
    .bind(report_id.parse::<Uuid>().expect("report ID is a UUID"))
    .fetch_one(&database)
    .await
    .expect("report resolution audit entry should exist");
    assert_eq!(audit_count, 1);
}

async fn create_administrator_session(database: &PgPool, role_key: &str) -> (Uuid, String) {
    let administrator_id = Uuid::new_v4();
    let token = one_time_token::create_one_time_token();
    let token_hash = one_time_token::hash_one_time_token(&token);
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at)
        VALUES ($1, $2, 'integration-test-password-hash', 'test-secret-ciphertext', now())",
    )
    .bind(administrator_id)
    .bind(format!("{administrator_id}@example.test"))
    .execute(database)
    .await
    .expect("test administrator should be inserted");
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(role_key)
    .bind(
        peerfora_api::constants::admin_auth::SEEDED_ROOT_ADMINISTRATOR_ID
            .parse::<Uuid>()
            .expect("root administrator ID should be valid"),
    )
    .execute(database)
    .await
    .expect("test administrator role should be assigned");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(token_hash)
    .execute(database)
    .await
    .expect("test administrator session should be inserted");
    (administrator_id, token)
}

async fn send_admin_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> axum::response::Response {
    use axum::{body::Body, http::Request, http::header};
    use tower::ServiceExt;

    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::COOKIE, format!("peerfora_admin_session={token}"))
        .header(header::ORIGIN, TEST_ORIGIN);
    let body = if let Some(body) = body {
        request = request.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&body).expect("request body should serialize"))
    } else {
        Body::empty()
    };
    app.clone()
        .oneshot(request.body(body).expect("request should build"))
        .await
        .expect("router should return a response")
}
