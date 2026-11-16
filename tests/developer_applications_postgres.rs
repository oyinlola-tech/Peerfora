#[path = "developer_applications_postgres/support.rs"]
mod support;

use axum::http::StatusCode;
use peerfora_api::{app::build_app, database::connect_database, utils::one_time_token};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{create_account_fixture, response_json, send_json_request, test_app_state};
use uuid::Uuid;

#[tokio::test]
async fn developer_applications_use_human_sessions_permissions_and_review_audit() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let account = create_account_fixture(&database, "developer").await;
    let app = build_app(test_app_state(database.clone(), database_url));
    let profile = send_json_request(
        &app,
        "POST",
        "/api/v1/developers/me",
        &account.session_token,
        json!({"displayName":"Research Tools","description":"Academic integrations."}),
    )
    .await;
    assert_eq!(profile.status(), StatusCode::CREATED);

    let registration = send_json_request(
        &app,
        "POST",
        "/api/v1/developers/applications",
        &account.session_token,
        json!({
            "name":"Directory Reader",
            "description":"Reads public academic directory records.",
            "applicationType":"web",
            "requestedScopes":["academic_directory_read"]
        }),
    )
    .await;
    let registration_status = registration.status();
    let application: Value = response_json(registration).await;
    assert_eq!(
        registration_status,
        StatusCode::CREATED,
        "registration response: {application}"
    );
    let application_id = application["id"].as_str().expect("application ID");
    assert_eq!(application["applicationStatus"], "draft");
    assert_eq!(application["scopes"][0]["status"], "requested");
    let draft_credential = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/developers/applications/{application_id}/credentials"),
        &account.session_token,
        json!({"name":"Not Approved"}),
    )
    .await;
    assert_eq!(draft_credential.status(), StatusCode::CONFLICT);
    let owner_applications = send_json_request(
        &app,
        "GET",
        "/api/v1/developers/applications",
        &account.session_token,
        json!({}),
    )
    .await;
    assert_eq!(owner_applications.status(), StatusCode::OK);
    assert_eq!(
        response_json(owner_applications).await["items"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );

    let duplicate_scope = send_json_request(
        &app,
        "POST",
        "/api/v1/developers/applications",
        &account.session_token,
        json!({
            "name":"Duplicate Scope",
            "applicationType":"web",
            "requestedScopes":["academic_directory_read","academic_directory_read"]
        }),
    )
    .await;
    assert_eq!(duplicate_scope.status(), StatusCode::BAD_REQUEST);

    let submitted = send_json_request(
        &app,
        "POST",
        &format!("/api/v1/developers/applications/{application_id}/submit"),
        &account.session_token,
        json!({}),
    )
    .await;
    assert_eq!(submitted.status(), StatusCode::NO_CONTENT);

    let (_, administrator_token) =
        create_administrator_session(&database, "developer_application_reviewer").await;
    let student_cannot_review = send_json_request(
        &app,
        "GET",
        "/api/v1/admin/developers/applications?status=submitted",
        &account.session_token,
        json!({}),
    )
    .await;
    assert_eq!(student_cannot_review.status(), StatusCode::UNAUTHORIZED);

    let listing = send_admin_request(
        &app,
        "/api/v1/admin/developers/applications?status=submitted",
        &administrator_token,
    )
    .await;
    assert_eq!(listing.status(), StatusCode::OK);
    assert_eq!(response_json(listing).await["totalCount"], 1);

    let (_, support_agent_token) = create_administrator_session(&database, "support_agent").await;
    let unauthorized_review = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/developers/applications/{application_id}/review"),
        &support_agent_token,
        json!({"decision":"approve","reason":"This role must not review applications."}),
    )
    .await;
    assert_eq!(unauthorized_review.status(), StatusCode::FORBIDDEN);

    let reviewed = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/developers/applications/{application_id}/review"),
        &administrator_token,
        json!({"decision":"approve","reason":"The requested public directory access is appropriate."}),
    )
    .await;
    assert_eq!(reviewed.status(), StatusCode::NO_CONTENT);
    let reviewed_scopes: Vec<(String, String)> = sqlx::query_as(
        "SELECT application.application_status, scope.scope_status
         FROM developer_applications application
         JOIN developer_application_requested_scopes scope
             ON scope.application_id = application.id
         WHERE application.id = $1",
    )
    .bind(application_id.parse::<Uuid>().expect("application UUID"))
    .fetch_all(&database)
    .await
    .expect("review state should persist");
    assert_eq!(
        reviewed_scopes,
        vec![("approved".into(), "approved".into())]
    );
    let audit_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM administrative_audit_logs
         WHERE target_id = $1 AND action = 'developers.application.approved')",
    )
    .bind(application_id.parse::<Uuid>().expect("application UUID"))
    .fetch_one(&database)
    .await
    .expect("approval audit entry should exist");
    assert!(audit_exists);
}

async fn create_administrator_session(database: &PgPool, role_key: &str) -> (Uuid, String) {
    let administrator_id = Uuid::new_v4();
    let token = one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at)
         VALUES ($1, $2, 'integration-test-hash', 'test-ciphertext', now())",
    )
    .bind(administrator_id)
    .bind(format!("{administrator_id}@example.test"))
    .execute(database)
    .await
    .expect("administrator should be inserted");
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
            .expect("seeded root ID should be valid"),
    )
    .execute(database)
    .await
    .expect("reviewer role should be assigned");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
         VALUES ($1, $2, $3, now() + interval '1 hour')",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(one_time_token::hash_one_time_token(&token))
    .execute(database)
    .await
    .expect("administrator session should be inserted");
    (administrator_id, token)
}

async fn send_admin_request(
    app: &axum::Router,
    path: &str,
    token: &str,
) -> axum::response::Response {
    use axum::{
        body::Body,
        http::{Request, header},
    };
    use tower::ServiceExt;
    app.clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header(header::COOKIE, format!("peerfora_admin_session={token}"))
                .body(Body::empty())
                .expect("admin request should build"),
        )
        .await
        .expect("router should return response")
}

async fn send_admin_json_request(
    app: &axum::Router,
    path: &str,
    token: &str,
    body: Value,
) -> axum::response::Response {
    use axum::{
        body::Body,
        http::{Request, header},
    };
    use tower::ServiceExt;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::COOKIE, format!("peerfora_admin_session={token}"))
                .header(header::ORIGIN, support::TEST_ORIGIN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&body).expect("body should serialize"),
                ))
                .expect("admin request should build"),
        )
        .await
        .expect("router should return response")
}

#[path = "developer_applications_postgres/credential_suite.rs"]
mod credential_suite;
