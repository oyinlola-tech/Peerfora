use std::collections::HashMap;

use axum::http::HeaderValue;
use peerfora_api::{configs::settings::Settings, models::app_state::AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

pub const TEST_ORIGIN: &str = "https://peerfora.example.test";

pub struct AccountFixture {
    pub session_token: String,
}

pub async fn create_account_fixture(database: &PgPool, username_prefix: &str) -> AccountFixture {
    let account_id = Uuid::new_v4();
    let country_code: String = sqlx::query_scalar(
        "SELECT country_code::text FROM countries WHERE is_active = TRUE LIMIT 1",
    )
    .fetch_one(database)
    .await
    .expect("country reference data should exist");
    let username = format!("{username_prefix}_{}", account_id.simple());
    sqlx::query(
        "INSERT INTO accounts (
            id, email, username, password_hash, eligibility_country_code,
            age_policy_version, terms_accepted_at, email_verified_at
        ) VALUES ($1, $2, $3, 'integration-test-hash', $4, 'test-policy-v1', now(), now())",
    )
    .bind(account_id)
    .bind(format!("{username}@example.test"))
    .bind(username)
    .bind(country_code)
    .execute(database)
    .await
    .expect("verified account should be inserted");
    let session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    let token_hash = peerfora_api::utils::one_time_token::hash_one_time_token(&session_token);
    sqlx::query(
        "INSERT INTO account_sessions (account_id, token_hash, expires_at)
        VALUES ($1, $2, now() + interval '1 hour')",
    )
    .bind(account_id)
    .bind(token_hash)
    .execute(database)
    .await
    .expect("account session should be inserted");
    AccountFixture { session_token }
}

pub fn test_app_state(database: PgPool, database_url: String) -> AppState {
    let settings = Settings {
        host: "127.0.0.1".parse().expect("loopback address should parse"),
        port: 0,
        log_filter: "warn".to_owned(),
        database_url,
        cookie_secure: true,
        production_environment: false,
        admin_mfa_encryption_key: None,
        authentication_rate_limit_hash_key: "test-auth-rate-limit-key-long-enough-to-use"
            .to_owned(),
        allowed_origins: vec![
            TEST_ORIGIN
                .parse::<HeaderValue>()
                .expect("test origin should parse"),
        ],
        request_timeout_seconds: 30,
        country_policies: HashMap::new(),
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        smtp_from: None,
        media_storage_base_path: "./target/phase-four-test-media".to_owned(),
        public_url: TEST_ORIGIN.to_owned(),
        google_oidc: None,
        apple_oidc: None,
    };
    AppState::from_settings(database, &settings, None)
}

pub async fn create_public_community(app: &axum::Router, token: &str) -> String {
    let response = send_json_request(
        app,
        "POST",
        "/api/v1/communities",
        token,
        json!({
            "slug": format!("moderation-{}", Uuid::new_v4().simple()),
            "displayName": "Moderation test community",
            "category": "general",
            "visibility": "public"
        }),
    )
    .await;
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    response_json(response).await["slug"]
        .as_str()
        .expect("community should include a slug")
        .to_owned()
}

pub async fn create_community_post(app: &axum::Router, community_slug: &str, token: &str) -> Uuid {
    let response = send_json_request(
        app,
        "POST",
        &format!("/api/v1/communities/{community_slug}/posts"),
        token,
        json!({
            "kind": "discussion",
            "title": "Moderation report test post",
            "body": "A public post used by the moderation report integration test."
        }),
    )
    .await;
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    response_json(response).await["id"]
        .as_str()
        .expect("post should include an ID")
        .parse()
        .expect("post ID should be a UUID")
}

pub async fn send_json_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Value,
) -> axum::response::Response {
    use axum::{body::Body, http::Request, http::header};
    use tower::ServiceExt;

    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(
            serde_json::to_vec(&body).expect("request body should serialize"),
        ))
        .expect("request should build");
    app.clone()
        .oneshot(request)
        .await
        .expect("application should respond")
}

pub async fn response_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&bytes).expect("response body should be JSON")
}
