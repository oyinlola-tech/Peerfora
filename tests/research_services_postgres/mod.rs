use std::collections::HashMap;

use axum::http::HeaderValue;
use peerfora_api::{configs::settings::Settings, models::app_state::AppState};
use sqlx::PgPool;
use uuid::Uuid;

const TEST_PUBLIC_URL: &str = "https://peerfora.example.test";

pub struct AccountFixture {
    pub username: String,
    pub session_token: String,
}

pub async fn create_account_fixture(database: &PgPool, username_prefix: &str) -> AccountFixture {
    let account_id = Uuid::new_v4();
    let country_code: String = sqlx::query_scalar(
        "SELECT country_code::text FROM countries WHERE is_active = TRUE LIMIT 1",
    )
    .fetch_one(database)
    .await
    .expect("the country snapshot should be available");
    let username = format!(
        "{username_prefix}_{}",
        &account_id.simple().to_string()[..8]
    );
    let email = format!("{username}@example.test");
    sqlx::query(
        "INSERT INTO accounts (
            id, email, username, password_hash, eligibility_country_code,
            age_policy_version, terms_accepted_at, email_verified_at
        ) VALUES ($1, $2, $3, 'test-only-password-hash', $4, 'test-policy-v1', now(), now())",
    )
    .bind(account_id)
    .bind(email)
    .bind(&username)
    .bind(country_code)
    .execute(database)
    .await
    .expect("verified human account should persist");

    let session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    let session_hash = peerfora_api::utils::one_time_token::hash_one_time_token(&session_token);
    sqlx::query(
        "INSERT INTO account_sessions (account_id, token_hash, expires_at)
        VALUES ($1, $2, now() + interval '1 hour')",
    )
    .bind(account_id)
    .bind(session_hash)
    .execute(database)
    .await
    .expect("session should persist");

    AccountFixture {
        username,
        session_token,
    }
}

pub fn database_url_from_environment() -> String {
    std::env::var("PEERFORA_TEST_DATABASE_URL").expect("test checked the database URL")
}

pub fn test_app_state(database: PgPool, database_url: String) -> AppState {
    let settings = Settings {
        host: "127.0.0.1".parse().expect("loopback address is valid"),
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
                .parse::<HeaderValue>()
                .expect("test origin is a valid header"),
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
