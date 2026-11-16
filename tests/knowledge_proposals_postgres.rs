#[path = "developer_applications_postgres/support.rs"]
mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use peerfora_api::{app::build_app, database::connect_database, utils::one_time_token};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use support::{create_account_fixture, response_json, send_json_request, test_app_state};

#[tokio::test]
async fn knowledge_proposal_requires_editorial_approval_and_publication() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("disposable PostgreSQL should apply migrations");
    let contributor = create_account_fixture(&database, "knowledge_contributor").await;
    let subject_id: Uuid =
        sqlx::query_scalar("SELECT id FROM academic_subjects WHERE is_active ORDER BY id LIMIT 1")
            .fetch_one(&database)
            .await
            .expect("active academic subject should be seeded");
    let app = build_app(test_app_state(database.clone(), database_url));
    let invalid_source = submit_proposal(
        &app,
        &contributor.session_token,
        subject_id,
        "http://journal.example.test/paper",
    )
    .await;
    assert_eq!(invalid_source.status(), StatusCode::BAD_REQUEST);

    let proposal = submit_proposal(
        &app,
        &contributor.session_token,
        subject_id,
        "https://journal.example.test/paper",
    )
    .await;
    assert_eq!(proposal.status(), StatusCode::ACCEPTED);
    let proposal: Value = response_json(proposal).await;
    assert_eq!(proposal["status"], "submitted");
    let entry_id = proposal["id"].as_str().expect("proposal ID");

    let unavailable_before_review = request_public_entry(&app, entry_id).await;
    assert_eq!(unavailable_before_review.status(), StatusCode::NOT_FOUND);
    let (_, support_token) = create_administrator_session(&database, "support_agent").await;
    let (_, editor_token) = create_administrator_session(&database, "knowledge_editor").await;
    let queue = request_admin(
        &app,
        "GET",
        "/api/v1/admin/knowledge/proposals?limit=20&offset=0",
        &editor_token,
        None,
    )
    .await;
    assert_eq!(queue.status(), StatusCode::OK);
    let queue: Value = response_json(queue).await;
    assert!(
        queue["items"]
            .as_array()
            .is_some_and(|items| { items.iter().any(|item| item["id"] == entry_id) })
    );
    let support_denied = request_admin(
        &app,
        "GET",
        "/api/v1/admin/knowledge/proposals",
        &support_token,
        None,
    )
    .await;
    assert_eq!(support_denied.status(), StatusCode::FORBIDDEN);

    let review_path = format!("/api/v1/admin/knowledge/proposals/{entry_id}/review");
    let review = request_admin(
        &app,
        "POST",
        &review_path,
        &editor_token,
        Some(json!({"decision":"approve", "reason":"The proposal has an appropriate citation."})),
    )
    .await;
    assert_eq!(review.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        request_public_entry(&app, entry_id).await.status(),
        StatusCode::NOT_FOUND
    );

    let publish_path = format!("/api/v1/admin/knowledge/entries/{entry_id}/publish");
    let publish = request_admin(
        &app,
        "POST",
        &publish_path,
        &editor_token,
        Some(json!({"reason":"The approved entry is ready for public discovery."})),
    )
    .await;
    assert_eq!(publish.status(), StatusCode::NO_CONTENT);
    let public_entry = request_public_entry(&app, entry_id).await;
    assert_eq!(public_entry.status(), StatusCode::OK);
    let public_entry: Value = response_json(public_entry).await;
    assert_eq!(
        public_entry["citations"][0]["provenance"],
        "user_submitted_unverified"
    );
    assert_eq!(
        public_entry["citations"][0]["externalUrl"],
        "https://journal.example.test/paper"
    );
    let public_search = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/knowledge/entries?query=global%20learning")
                .body(Body::empty())
                .expect("Knowledge search request should build"),
        )
        .await
        .expect("Knowledge search should respond");
    assert_eq!(public_search.status(), StatusCode::OK);
    let public_search: Value = response_json(public_search).await;
    assert!(
        public_search["items"]
            .as_array()
            .is_some_and(|items| { items.iter().any(|item| item["id"] == entry_id) })
    );
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrative_audit_logs
        WHERE target_id = $1 AND action LIKE 'knowledge.%'",
    )
    .bind(entry_id.parse::<Uuid>().expect("entry UUID"))
    .fetch_one(&database)
    .await
    .expect("editorial actions should be audited");
    assert_eq!(audit_count, 2);
}

async fn submit_proposal(
    app: &axum::Router,
    token: &str,
    subject_id: Uuid,
    external_url: &str,
) -> axum::response::Response {
    send_json_request(
        app,
        "POST",
        "/api/v1/knowledge/proposals",
        token,
        json!({
            "academicSubjectId":subject_id,
            "title":"Global learning access evidence",
            "summary":"This summary describes a sourced knowledge proposal awaiting human editorial review.",
            "citations":[{
                "title":"A synthetic learning access paper",
                "doi":"10.5555/peerfora.knowledge.test",
                "externalUrl":external_url,
                "authors":["Researcher One"],
                "publicationYear":2024,
                "license":"CC BY 4.0"
            }]
        }),
    )
    .await
}

async fn request_public_entry(app: &axum::Router, entry_id: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/knowledge/entries/{entry_id}"))
                .body(Body::empty())
                .expect("Knowledge entry request should build"),
        )
        .await
        .expect("Knowledge entry should respond")
}

async fn request_admin(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::COOKIE, format!("peerfora_admin_session={token}"))
        .header(header::ORIGIN, support::TEST_ORIGIN);
    if body.is_some() {
        request = request.header(header::CONTENT_TYPE, "application/json");
    }
    app.clone()
        .oneshot(
            request
                .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .expect("editorial request should build"),
        )
        .await
        .expect("administrative Knowledge route should respond")
}

async fn create_administrator_session(database: &PgPool, role_key: &str) -> (Uuid, String) {
    let administrator_id = Uuid::new_v4();
    let token = one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrators
            (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at)
        VALUES ($1, $2, 'integration-test-hash', 'test-ciphertext', now())",
    )
    .bind(administrator_id)
    .bind(format!("{administrator_id}@example.test"))
    .execute(database)
    .await
    .expect("administrator fixture should persist");
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
            .expect("root ID"),
    )
    .execute(database)
    .await
    .expect("administrator role should persist");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '1 hour')",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(one_time_token::hash_one_time_token(&token))
    .execute(database)
    .await
    .expect("administrator session should persist");
    (administrator_id, token)
}
