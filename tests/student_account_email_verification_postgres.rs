use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use peerfora_api::{
    app::build_app,
    configs::settings::Settings,
    database::connect_database,
    enums::account_auth_error::AccountAuthError,
    interfaces::email_sender::{EmailMessage, EmailSender},
    models::app_state::AppState,
    repositories::account_repository::{self, NewAccountRecord},
    utils::one_time_token::hash_one_time_token,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Default)]
struct TestSender {
    fail: AtomicBool,
    messages: Mutex<Vec<String>>,
}

#[async_trait]
impl EmailSender for TestSender {
    async fn send_email(
        &self,
        _recipient: &str,
        message: &EmailMessage,
    ) -> Result<(), AccountAuthError> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(AccountAuthError::EmailDeliveryFailed);
        }
        self.messages
            .lock()
            .unwrap()
            .push(message.plain_text.clone());
        Ok(())
    }
}

fn app_state(pool: PgPool, database_url: String, sender: Option<Arc<dyn EmailSender>>) -> AppState {
    let settings = Settings {
        host: "127.0.0.1".parse().unwrap(),
        port: 0,
        log_filter: "warn".to_owned(),
        database_url,
        cookie_secure: true,
        production_environment: false,
        admin_mfa_encryption_key: None,
        authentication_rate_limit_hash_key: "test-auth-rate-limit-key-long-enough-to-use"
            .to_owned(),
        allowed_origins: vec!["https://peerfora.example.test".parse().unwrap()],
        request_timeout_seconds: 30,
        country_policies: HashMap::new(),
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        smtp_from: None,
        public_url: "https://peerfora.example.test".to_owned(),
        google_oidc: None,
        apple_oidc: None,
    };
    AppState::from_settings(pool, &settings, sender)
}

async fn request(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Value,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn submit_verification(app: &axum::Router, token: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/verify-email")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(format!("token={token}")))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn response_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap()
}

#[tokio::test]
async fn registration_recovery_and_confirmation_are_safe() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let pool = connect_database(&database_url).await.unwrap();
    let sender = Arc::new(TestSender::default());
    let app = build_app(app_state(
        pool.clone(),
        database_url.clone(),
        Some(sender.clone()),
    ));
    let no_sender = build_app(app_state(pool.clone(), database_url, None));
    let id = Uuid::new_v4().simple().to_string();
    let underage = request(
        &app,
        "POST",
        "/api/v1/auth/register",
        json!({
            "email": format!("underage-{id}@example.test"),
            "username": format!("u_{}", &id[..16]),
            "password": "a sufficiently long password", "dateOfBirth": "2990-01-01",
            "countryCode": "NG", "acceptedTerms": true
        }),
    )
    .await;
    assert_eq!(underage.status(), StatusCode::FORBIDDEN);
    let email = format!("verification-{id}@example.test");
    let username = format!("v_{}", &id[..16]);
    let registration = json!({
        "email": email, "username": username, "password": "a sufficiently long password",
        "dateOfBirth": "1990-01-01", "countryCode": "NG", "acceptedTerms": true
    });
    assert_eq!(
        request(
            &no_sender,
            "POST",
            "/api/v1/auth/register",
            registration.clone()
        )
        .await
        .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);

    sender.fail.store(true, Ordering::SeqCst);
    assert_eq!(
        request(&app, "POST", "/api/v1/auth/register", registration)
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let tokens: Vec<String> = sqlx::query_scalar(
        "SELECT token.token_hash FROM email_verification_tokens token JOIN accounts account ON account.id = token.account_id WHERE account.email = $1",
    ).bind(&email).fetch_all(&pool).await.unwrap();
    assert_eq!(tokens.len(), 1);

    let path = "/api/v1/auth/verify-email/request";
    let failed_delivery = request(&app, "POST", path, json!({"email": email})).await;
    assert_eq!(failed_delivery.status(), StatusCode::ACCEPTED);
    let generic = response_body(failed_delivery).await;
    sender.fail.store(false, Ordering::SeqCst);
    let resend = request(&app, "POST", path, json!({"email": email.to_uppercase()})).await;
    assert_eq!(resend.status(), StatusCode::ACCEPTED);
    assert_eq!(response_body(resend).await, generic);
    let message = sender.messages.lock().unwrap()[0].clone();
    let token = message
        .lines()
        .find_map(|line| line.split_once("?token=").map(|(_, token)| token))
        .expect("verification link should contain a token")
        .to_owned();
    let hash = hash_one_time_token(&token);
    assert_ne!(token, hash);
    let stored: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM email_verification_tokens WHERE token_hash = $1)",
    )
    .bind(&hash)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(stored);
    let verify_path = format!("/api/v1/auth/verify-email?token={token}");
    let preview = request(&app, "GET", &verify_path, json!({})).await;
    assert_eq!(preview.status(), StatusCode::OK);
    let preview_body = to_bytes(preview.into_body(), 4096).await.unwrap();
    let preview_html = std::str::from_utf8(&preview_body).unwrap();
    assert!(!preview_html.contains(&token));
    assert!(preview_html.contains("action=\"\""));
    let conflicting = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&verify_path)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(format!("token={}", "a".repeat(64))))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conflicting.status(), StatusCode::BAD_REQUEST);
    let still_unverified: bool =
        sqlx::query_scalar("SELECT email_verified_at IS NULL FROM accounts WHERE email = $1")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(still_unverified);
    let (first, second) = tokio::join!(
        submit_verification(&app, &token),
        app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(&verify_path)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::empty())
                .unwrap()
        ),
    );
    let statuses = [first.status(), second.unwrap().status()];
    assert!(statuses.contains(&StatusCode::OK) && statuses.contains(&StatusCode::BAD_REQUEST));
    let remaining: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM email_verification_tokens token JOIN accounts account ON account.id = token.account_id WHERE account.email = $1",
    ).bind(&email).fetch_one(&pool).await.unwrap();
    assert_eq!(remaining, 0);
    let unknown = format!("unknown-{}@example.test", Uuid::new_v4());
    let unknown_response = request(&app, "POST", path, json!({"email": unknown})).await;
    assert_eq!(unknown_response.status(), StatusCode::ACCEPTED);
    assert_eq!(response_body(unknown_response).await, generic);
    let verified = request(&app, "POST", path, json!({"email": email})).await;
    assert_eq!(response_body(verified).await, generic);
    assert_eq!(sender.messages.lock().unwrap().len(), 1);
    let limited = request(&app, "POST", path, json!({"email": email})).await;
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    let login = request(
        &app,
        "POST",
        "/api/v1/auth/login",
        json!({"email": email, "password": "a sufficiently long password"}),
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);

    sender.fail.store(true, Ordering::SeqCst);
    let reset_path = "/api/v1/auth/password-reset/request";
    let known_reset = request(&app, "POST", reset_path, json!({"email": email})).await;
    assert_eq!(known_reset.status(), StatusCode::ACCEPTED);
    let unknown_reset = request(&app, "POST", reset_path, json!({"email": unknown})).await;
    assert_eq!(unknown_reset.status(), StatusCode::ACCEPTED);
    assert_eq!(
        response_body(known_reset).await,
        response_body(unknown_reset).await
    );
    let unconfigured_reset = request(&no_sender, "POST", reset_path, json!({"email": email})).await;
    assert_eq!(unconfigured_reset.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn account_and_confirmation_token_are_inserted_atomically() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let pool = connect_database(&database_url).await.unwrap();
    let id = Uuid::new_v4().simple().to_string();
    let first_email = format!("first-{id}@example.test");
    let second_email = format!("second-{id}@example.test");
    let first_username = format!("a_{}", &id[..16]);
    let second_username = format!("b_{}", &id[..16]);
    let duplicate_hash = format!("test-{id}");
    let first = NewAccountRecord {
        email: &first_email,
        username: &first_username,
        display_name: None,
        password_hash: "test-hash",
        country_code: "NG",
        age_policy_version: "test",
        verification_token_hash: &duplicate_hash,
    };
    account_repository::create_account(&pool, first)
        .await
        .unwrap();
    let second = NewAccountRecord {
        email: &second_email,
        username: &second_username,
        display_name: None,
        password_hash: "test-hash",
        country_code: "NG",
        age_policy_version: "test",
        verification_token_hash: &duplicate_hash,
    };
    assert!(
        account_repository::create_account(&pool, second)
            .await
            .is_err()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM accounts WHERE email = $1")
        .bind(&second_email)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn suspended_accounts_and_expired_tokens_cannot_be_verified() {
    let Ok(database_url) = std::env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let pool = connect_database(&database_url).await.unwrap();
    let sender = Arc::new(TestSender::default());
    let app = build_app(app_state(pool.clone(), database_url, Some(sender.clone())));
    let id = Uuid::new_v4().simple().to_string();
    let email = format!("suspended-{id}@example.test");
    let username = format!("s_{}", &id[..16]);
    let account_id: Uuid = sqlx::query_scalar(
        "INSERT INTO accounts (email, username, password_hash, eligibility_country_code,
         age_policy_version, terms_accepted_at, account_status)
         VALUES ($1, $2, 'test-hash', 'NG', 'test', now(), 'suspended') RETURNING id",
    )
    .bind(&email)
    .bind(username)
    .fetch_one(&pool)
    .await
    .unwrap();
    let token = format!("{id}{id}");
    sqlx::query("INSERT INTO email_verification_tokens (account_id, token_hash, expires_at) VALUES ($1, $2, now() + interval '1 hour')")
        .bind(account_id)
        .bind(hash_one_time_token(&token))
        .execute(&pool)
        .await
        .unwrap();
    let resend = request(
        &app,
        "POST",
        "/api/v1/auth/verify-email/request",
        json!({"email": email}),
    )
    .await;
    assert_eq!(resend.status(), StatusCode::ACCEPTED);
    assert!(sender.messages.lock().unwrap().is_empty());
    assert_eq!(
        submit_verification(&app, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
    sqlx::query("UPDATE accounts SET account_status = 'active' WHERE id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE email_verification_tokens SET expires_at = now() - interval '1 minute' WHERE account_id = $1")
        .bind(account_id).execute(&pool).await.unwrap();
    assert_eq!(
        submit_verification(&app, &token).await.status(),
        StatusCode::BAD_REQUEST
    );
    let verified: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT email_verified_at FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(verified.is_none());
}
