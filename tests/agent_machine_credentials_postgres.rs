#[path = "developer_applications_postgres/support.rs"]
mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use peerfora_api::{
    app::build_app, constants::admin_auth, database::connect_database, utils::one_time_token,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{create_account_fixture, response_json, send_json_request, test_app_state};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn approved_agents_receive_capability_bound_machine_credentials() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("disposable PostgreSQL should apply all migrations");
    let operator = create_account_fixture(&database, "agent_credential_owner").await;
    let app = build_app(test_app_state(database.clone(), database_url));
    let subject_id: Uuid =
        sqlx::query_scalar("SELECT id FROM academic_subjects WHERE is_active = TRUE LIMIT 1")
            .fetch_one(&database)
            .await
            .expect("seeded subject should exist");
    let registration = send_json_request(
        &app,
        "POST",
        "/api/v1/agents",
        &operator.session_token,
        json!({
            "handle":"credential_agent",
            "displayName":"Credential Agent",
            "specializationSubjectIds":[subject_id],
            "requestedCapabilities":["read_public_content", "research"]
        }),
    )
    .await;
    assert_eq!(registration.status(), StatusCode::CREATED);
    let registration: Value = response_json(registration).await;
    let agent_id = registration["id"].as_str().expect("agent ID");

    let pending_issue = send_json_request(
        &app,
        "POST",
        &credential_collection_path(agent_id),
        &operator.session_token,
        json!({"name":"Research credential", "capabilities":["research"]}),
    )
    .await;
    assert_eq!(pending_issue.status(), StatusCode::CONFLICT);

    let (_, reviewer_token) = create_agent_reviewer_session(&database).await;
    let review = send_admin_json_request(
        &app,
        &format!("/api/v1/admin/agents/{agent_id}/review"),
        &reviewer_token,
        json!({
            "decision":"approve",
            "approvedCapabilities":["read_public_content", "research"],
            "reason":"The agent has an accountable operator and a bounded public-data use case."
        }),
    )
    .await;
    assert_eq!(review.status(), StatusCode::NO_CONTENT);

    let issue = send_json_request(
        &app,
        "POST",
        &credential_collection_path(agent_id),
        &operator.session_token,
        json!({
            "name":"Subject directory reader",
            "capabilities":["read_public_content"],
            "expiresInDays":45
        }),
    )
    .await;
    assert_eq!(issue.status(), StatusCode::CREATED);
    assert_eq!(issue.headers()[header::CACHE_CONTROL], "no-store");
    let issue: Value = response_json(issue).await;
    let first_credential_id = issue["credential"]["id"].as_str().expect("credential ID");
    let first_secret = issue["secret"].as_str().expect("one-time secret");
    assert!(first_secret.starts_with("pfagent_"));
    assert_eq!(
        issue["credential"]["capabilities"],
        json!(["read_public_content"])
    );
    let stored_hash: String =
        sqlx::query_scalar("SELECT token_hash FROM agent_machine_credentials WHERE id = $1")
            .bind(
                first_credential_id
                    .parse::<Uuid>()
                    .expect("credential UUID"),
            )
            .fetch_one(&database)
            .await
            .expect("credential hash should persist");
    assert_ne!(stored_hash, first_secret);

    let agent_subjects = send_agent_request(&app, first_secret).await;
    assert_eq!(agent_subjects.status(), StatusCode::OK);
    let student_token = send_agent_request(&app, &operator.session_token).await;
    assert_eq!(student_token.status(), StatusCode::UNAUTHORIZED);

    let credentials = send_json_request(
        &app,
        "GET",
        &credential_collection_path(agent_id),
        &operator.session_token,
        json!({}),
    )
    .await;
    assert_eq!(credentials.status(), StatusCode::OK);
    let credentials: Value = response_json(credentials).await;
    assert_eq!(credentials["items"].as_array().map(Vec::len), Some(1));
    assert!(credentials["items"][0].get("secret").is_none());

    let rotated = send_json_request(
        &app,
        "POST",
        &format!(
            "{}/rotate",
            credential_item_path(agent_id, first_credential_id)
        ),
        &operator.session_token,
        json!({"expiresInDays":30}),
    )
    .await;
    assert_eq!(rotated.status(), StatusCode::CREATED);
    let rotated: Value = response_json(rotated).await;
    let second_credential_id = rotated["credential"]["id"].as_str().expect("new ID");
    let second_secret = rotated["secret"].as_str().expect("rotated secret");
    assert_eq!(
        rotated["credential"]["capabilities"],
        json!(["read_public_content"])
    );
    assert_eq!(
        send_agent_request(&app, first_secret).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send_agent_request(&app, second_secret).await.status(),
        StatusCode::OK
    );

    sqlx::query(
        "UPDATE agent_capability_requests SET request_status = 'revoked'
         WHERE agent_actor_id = $1 AND capability_key = 'read_public_content'",
    )
    .bind(agent_id.parse::<Uuid>().expect("agent UUID"))
    .execute(&database)
    .await
    .expect("capability revocation should persist");
    assert_eq!(
        send_agent_request(&app, second_secret).await.status(),
        StatusCode::FORBIDDEN
    );

    let revoked = send_json_request(
        &app,
        "POST",
        &format!(
            "{}/revoke",
            credential_item_path(agent_id, second_credential_id)
        ),
        &operator.session_token,
        json!({}),
    )
    .await;
    assert_eq!(revoked.status(), StatusCode::NO_CONTENT);
    let stored_status: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM agent_machine_credentials WHERE id = $1")
            .bind(
                second_credential_id
                    .parse::<Uuid>()
                    .expect("credential UUID"),
            )
            .fetch_one(&database)
            .await
            .expect("credential revocation should persist");
    assert!(stored_status.is_some());
}

fn credential_collection_path(agent_id: &str) -> String {
    format!("/api/v1/agents/{agent_id}/credentials")
}

fn credential_item_path(agent_id: &str, credential_id: &str) -> String {
    format!("{}/{}", credential_collection_path(agent_id), credential_id)
}

async fn send_agent_request(app: &axum::Router, credential: &str) -> axum::response::Response {
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/agent-api/subjects?limit=5")
        .header(header::AUTHORIZATION, format!("Bearer {credential}"))
        .body(Body::empty())
        .expect("agent API request should build");
    app.clone()
        .oneshot(request)
        .await
        .expect("agent API should return a response")
}

async fn send_admin_json_request(
    app: &axum::Router,
    path: &str,
    session_token: &str,
    body: Value,
) -> axum::response::Response {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header(
            header::COOKIE,
            format!("{}={session_token}", admin_auth::ADMIN_SESSION_COOKIE_NAME),
        )
        .header(header::ORIGIN, support::TEST_ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("admin request should build");
    app.clone()
        .oneshot(request)
        .await
        .expect("admin API should return a response")
}

async fn create_agent_reviewer_session(database: &PgPool) -> (Uuid, String) {
    let administrator_id = Uuid::new_v4();
    let session_token = one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at)
         VALUES ($1, $2, 'integration-test-hash', 'test-ciphertext', now())",
    )
    .bind(administrator_id)
    .bind(format!("{administrator_id}@example.test"))
    .execute(database)
    .await
    .expect("agent reviewer should persist");
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
         VALUES ($1, $2, 'agent_reviewer', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(
        admin_auth::SEEDED_ROOT_ADMINISTRATOR_ID
            .parse::<Uuid>()
            .expect("seeded administrator ID should be valid"),
    )
    .execute(database)
    .await
    .expect("reviewer role should persist");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
         VALUES ($1, $2, $3, now() + interval '1 hour')",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(one_time_token::hash_one_time_token(&session_token))
    .execute(database)
    .await
    .expect("reviewer session should persist");
    (administrator_id, session_token)
}
