#[path = "research_services_postgres/mod.rs"]
mod research_services_postgres;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use peerfora_api::{app::build_app, database::connect_database};
use research_services_postgres::{
    create_account_fixture, database_url_from_environment, test_app_state,
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

const RESEARCH_PROJECTS_PATH: &str = "/api/v1/research/projects";
const TEST_PUBLIC_URL: &str = "https://peerfora.example.test";
const ADMIN_SESSION_COOKIE_NAME: &str = "peerfora_admin_session";

#[tokio::test]
async fn research_administration_enforces_permissions_and_audits_private_metadata() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let owner = create_account_fixture(&database, "admin_research_owner").await;
    let app = build_app(test_app_state(
        database.clone(),
        database_url_from_environment(),
    ));
    let project_id = create_private_project(&app, &owner.session_token).await;
    let (super_admin_id, super_admin_token) =
        create_administrator_session(&database, "super_administrator").await;
    let (_, limited_admin_token) =
        create_administrator_session(&database, "analytics_viewer").await;

    assert!(!owner.username.is_empty());
    let student_token = send_student_request(
        &app,
        "GET",
        "/api/v1/admin/research/projects",
        &owner.session_token,
    )
    .await;
    assert_eq!(student_token.status(), StatusCode::UNAUTHORIZED);

    let denied = send_administrator_request(
        &app,
        "GET",
        "/api/v1/admin/research/projects",
        &limited_admin_token,
        None,
    )
    .await;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    let search = send_administrator_request(
        &app,
        "GET",
        "/api/v1/admin/research/projects?search=private",
        &super_admin_token,
        None,
    )
    .await;
    assert_eq!(search.status(), StatusCode::OK);
    let result = response_json(search).await;
    assert_eq!(result["totalCount"], 1);
    assert_eq!(result["items"][0]["id"], project_id.to_string());
    assert!(result["items"][0].get("description").is_none());
    assert!(result["items"][0].get("objectives").is_none());

    let details = send_administrator_request(
        &app,
        "GET",
        &format!("/api/v1/admin/research/projects/{project_id}"),
        &super_admin_token,
        None,
    )
    .await;
    assert_eq!(details.status(), StatusCode::OK);

    let invalid_reason = send_administrator_request(
        &app,
        "POST",
        &format!("/api/v1/admin/research/projects/{project_id}/suspend"),
        &super_admin_token,
        Some(json!({"reasonCode": "unrecognized_reason"})),
    )
    .await;
    assert_eq!(invalid_reason.status(), StatusCode::BAD_REQUEST);

    let suspended = send_administrator_request(
        &app,
        "POST",
        &format!("/api/v1/admin/research/projects/{project_id}/suspend"),
        &super_admin_token,
        Some(json!({"reasonCode": "integrity_concern"})),
    )
    .await;
    assert_eq!(suspended.status(), StatusCode::NO_CONTENT);
    let hidden_from_owner = send_student_request(
        &app,
        "GET",
        &format!("{RESEARCH_PROJECTS_PATH}/{project_id}"),
        &owner.session_token,
    )
    .await;
    assert_eq!(hidden_from_owner.status(), StatusCode::NOT_FOUND);

    let restored = send_administrator_request(
        &app,
        "POST",
        &format!("/api/v1/admin/research/projects/{project_id}/restore"),
        &super_admin_token,
        Some(json!({"reasonCode": "other"})),
    )
    .await;
    assert_eq!(restored.status(), StatusCode::NO_CONTENT);
    let visible_to_owner = send_student_request(
        &app,
        "GET",
        &format!("{RESEARCH_PROJECTS_PATH}/{project_id}"),
        &owner.session_token,
    )
    .await;
    assert_eq!(visible_to_owner.status(), StatusCode::OK);

    let audit_events: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM administrative_audit_logs
        WHERE actor_administrator_id = $1 AND target_type = 'research_project'
            AND target_id = $2 ORDER BY created_at",
    )
    .bind(super_admin_id)
    .bind(project_id)
    .fetch_all(&database)
    .await
    .expect("research administration should be audited");
    assert_eq!(
        audit_events,
        [
            "research.project.metadata_read",
            "research.project.metadata_read",
            "research.project.suspended",
            "research.project.restored"
        ]
    );
}

async fn create_private_project(app: &axum::Router, token: &str) -> Uuid {
    let response = send_student_json(
        app,
        "POST",
        RESEARCH_PROJECTS_PATH,
        token,
        json!({"title": "Private administration project", "description": "Not returned by admin metadata."}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    response_json(response).await["id"]
        .as_str()
        .expect("project response should include an ID")
        .parse()
        .expect("project ID should be a UUID")
}

async fn create_administrator_session(database: &sqlx::PgPool, role_key: &str) -> (Uuid, String) {
    let administrator_id = Uuid::new_v4();
    let token = peerfora_api::utils::one_time_token::create_one_time_token();
    let token_hash = peerfora_api::utils::one_time_token::hash_one_time_token(&token);
    sqlx::query(
        "INSERT INTO administrators (
            id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at
        ) VALUES ($1, $2, 'integration-test-password-hash', 'integration-test-encrypted-secret', now())",
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
            .expect("root ID should be valid"),
    )
    .execute(database)
    .await
    .expect("test role should be assigned");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(token_hash)
    .execute(database)
    .await
    .expect("administrator session should be inserted");
    (administrator_id, token)
}

async fn send_administrator_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(
            header::COOKIE,
            format!("{ADMIN_SESSION_COOKIE_NAME}={token}"),
        )
        .header(header::ORIGIN, TEST_PUBLIC_URL);
    let body = if let Some(body) = body {
        request = request.header(header::CONTENT_TYPE, "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    app.clone()
        .oneshot(request.body(body).expect("request should be valid"))
        .await
        .expect("router should handle request")
}

async fn send_student_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
) -> axum::response::Response {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request should be valid");
    app.clone()
        .oneshot(request)
        .await
        .expect("router should handle request")
}

async fn send_student_json(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Value,
) -> axum::response::Response {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should be valid");
    app.clone()
        .oneshot(request)
        .await
        .expect("router should handle request")
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 2_000_000)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response should contain JSON")
}
