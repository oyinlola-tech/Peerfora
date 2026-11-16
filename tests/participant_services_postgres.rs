use std::collections::HashMap;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use peerfora_api::{
    app::build_app,
    configs::settings::Settings,
    database::connect_database,
    models::app_state::AppState,
    services::institutions::queries::search_institutions_query::search_institutions,
    utils::one_time_token::{create_one_time_token, hash_one_time_token},
};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_PUBLIC_URL: &str = "https://peerfora.example.test";

#[tokio::test]
async fn participant_identity_and_institution_services_use_postgresql() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL database should migrate");
    let fixture = create_verified_operator(&database).await;
    assert_human_account_has_typed_actor(&database, fixture.account_id).await;
    verify_agent_registration_and_operator_access(&database, &fixture).await;
    verify_institution_directory_search(&database).await;
}

struct AccountFixture {
    account_id: Uuid,
    session_token: String,
}

async fn create_verified_operator(database: &PgPool) -> AccountFixture {
    let account_id = Uuid::new_v4();
    let country_code: String = sqlx::query_scalar(
        "SELECT country_code::text FROM countries WHERE is_active = TRUE LIMIT 1",
    )
    .fetch_one(database)
    .await
    .expect("the country reference snapshot should be available");
    let email = format!("operator-{account_id}@example.test");
    let username = format!("operator_{}", &account_id.simple().to_string()[..12]);

    sqlx::query(
        "INSERT INTO accounts (
            id, email, username, password_hash, eligibility_country_code,
            age_policy_version, terms_accepted_at, email_verified_at
        ) VALUES ($1, $2, $3, 'test-only-password-hash', $4, 'test-policy-v1', now(), now())",
    )
    .bind(account_id)
    .bind(email)
    .bind(username)
    .bind(&country_code)
    .execute(database)
    .await
    .expect("verified human operator should persist");

    let session_token = create_one_time_token();
    sqlx::query(
        "INSERT INTO account_sessions (account_id, token_hash, expires_at)
        VALUES ($1, $2, now() + interval '1 hour')",
    )
    .bind(account_id)
    .bind(hash_one_time_token(&session_token))
    .execute(database)
    .await
    .expect("operator session should persist");

    AccountFixture {
        account_id,
        session_token,
    }
}

async fn assert_human_account_has_typed_actor(database: &PgPool, account_id: Uuid) {
    let actor_type: String = sqlx::query_scalar(
        "SELECT actor.actor_type
        FROM participant_actors actor
        JOIN human_actor_identities identity ON identity.actor_id = actor.id
        WHERE identity.account_id = $1",
    )
    .bind(account_id)
    .fetch_one(database)
    .await
    .expect("account creation should create its human actor");
    assert_eq!(actor_type, "human");
}

async fn verify_agent_registration_and_operator_access(
    database: &PgPool,
    fixture: &AccountFixture,
) {
    let subject_id: Uuid =
        sqlx::query_scalar("SELECT id FROM academic_subjects WHERE is_active = TRUE LIMIT 1")
            .fetch_one(database)
            .await
            .expect("starter academic subjects should be available");
    let app = build_app(test_app_state(
        database.clone(),
        database_url_from_environment(),
    ));

    let unauthenticated = send_json_request(
        app.clone(),
        "POST",
        "/api/v1/agents",
        None,
        agent_registration_body("atlas_research", subject_id),
    )
    .await;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let registration = send_json_request(
        app.clone(),
        "POST",
        "/api/v1/agents",
        Some(&fixture.session_token),
        agent_registration_body("atlas_research", subject_id),
    )
    .await;
    assert_eq!(registration.status(), StatusCode::CREATED);
    let registration_json: serde_json::Value = response_json(registration).await;
    let agent_id = registration_json["id"]
        .as_str()
        .expect("registered agent should return its actor ID")
        .parse::<Uuid>()
        .expect("registered agent ID should be a UUID");
    assert_eq!(registration_json["reviewStatus"], "pending_review");
    assert_eq!(registration_json["autonomousResearchEnabled"], false);
    assert_eq!(registration_json["autonomousPublishingEnabled"], false);
    assert!(
        registration_json["requestedCapabilities"]
            .as_array()
            .is_some_and(|capabilities| capabilities.contains(&json!("research")))
    );
    assert!(registration_json.get("operatorAccountId").is_none());

    let own_agents = send_request(
        app.clone(),
        "GET",
        "/api/v1/agents/me?limit=200",
        Some(&fixture.session_token),
    )
    .await;
    assert_eq!(own_agents.status(), StatusCode::OK);
    let own_agents_json: serde_json::Value = response_json(own_agents).await;
    assert_eq!(own_agents_json.as_array().map(Vec::len), Some(1));

    let duplicate_handle = send_json_request(
        app,
        "POST",
        "/api/v1/agents",
        Some(&fixture.session_token),
        agent_registration_body("atlas_research", subject_id),
    )
    .await;
    assert_eq!(duplicate_handle.status(), StatusCode::CONFLICT);

    let registration_total: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_actor_identities WHERE operator_account_id = $1",
    )
    .bind(fixture.account_id)
    .fetch_one(database)
    .await
    .expect("agent identities should be queryable");
    assert_eq!(registration_total, 1);

    let capability_status: String = sqlx::query_scalar(
        "SELECT request_status FROM agent_capability_requests
        WHERE agent_actor_id = $1 AND capability_key = 'research'",
    )
    .bind(agent_id)
    .fetch_one(database)
    .await
    .expect("requested capability should persist");
    assert_eq!(capability_status, "requested");
}

fn agent_registration_body(handle: &str, subject_id: Uuid) -> serde_json::Value {
    json!({
        "handle": handle,
        "displayName": "Atlas Research",
        "description": "Research agent profile used for a local integration test.",
        "modelDisclosure": "Test model",
        "specializationSubjectIds": [subject_id],
        "requestedCapabilities": ["research", "cite_sources"]
    })
}

async fn verify_institution_directory_search(database: &PgPool) {
    let institution_id = Uuid::new_v4();
    let search_name = format!("Peerfora Test Institute {institution_id}");
    let slug = format!("peerfora-test-{institution_id}");
    sqlx::query(
        "INSERT INTO institutions (
            id, slug, display_name, country_code, institution_type, record_source
        ) VALUES ($1, $2, $3, $4, 'university', 'local_integration_test')",
    )
    .bind(institution_id)
    .bind(slug)
    .bind(&search_name)
    .bind(&country_code_for_directory_test(database).await)
    .execute(database)
    .await
    .expect("institution directory fixture should persist");

    let results = search_institutions(database, Some(search_name.as_str()), Some(10))
        .await
        .expect("institution search should return stored directory data");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, institution_id);
}

async fn country_code_for_directory_test(database: &PgPool) -> String {
    sqlx::query_scalar("SELECT country_code::text FROM countries WHERE is_active = TRUE LIMIT 1")
        .fetch_one(database)
        .await
        .expect("active country should exist")
}

fn test_app_state(database: PgPool, database_url: String) -> AppState {
    let settings = Settings {
        host: "127.0.0.1".parse().expect("loopback address"),
        port: 0,
        log_filter: "warn".to_owned(),
        database_url,
        cookie_secure: true,
        production_environment: false,
        admin_mfa_encryption_key: None,
        authentication_rate_limit_hash_key: "integration-test-rate-limit-key-long-enough"
            .to_owned(),
        allowed_origins: vec![
            TEST_PUBLIC_URL
                .parse()
                .expect("test origin should be valid"),
        ],
        request_timeout_seconds: 30,
        country_policies: HashMap::new(),
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        smtp_from: None,
        public_url: TEST_PUBLIC_URL.to_owned(),
        google_oidc: None,
        apple_oidc: None,
    };
    AppState::from_settings(database, &settings, None)
}

fn database_url_from_environment() -> String {
    std::env::var("PEERFORA_TEST_DATABASE_URL").expect("database URL was checked by the test")
}

async fn send_json_request(
    app: axum::Router,
    method: &str,
    path: &str,
    session_token: Option<&str>,
    body: serde_json::Value,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = session_token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    app.oneshot(
        request
            .body(Body::from(body.to_string()))
            .expect("HTTP request should be valid"),
    )
    .await
    .expect("API route should return a response")
}

async fn send_request(
    app: axum::Router,
    method: &str,
    path: &str,
    session_token: Option<&str>,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = session_token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    app.oneshot(
        request
            .body(Body::empty())
            .expect("HTTP request should be valid"),
    )
    .await
    .expect("API route should return a response")
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("HTTP response body should be readable");
    serde_json::from_slice(&body).expect("HTTP response should be JSON")
}
