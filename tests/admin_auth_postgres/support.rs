use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use peerfora_api::{
    app::build_app, configs::settings::Settings, database::connect_database,
    enums::account_auth_error::AccountAuthError, interfaces::email_sender::EmailSender,
    services::admin_auth::commands::provision_first_administrator_command::provision_first_administrator,
};
use sqlx::PgPool;
use totp_rs::{Builder, Secret};
use tower::ServiceExt;

pub(super) const ADMIN_EMAIL: &str = "phase-three@example.test";
pub(super) const ADMIN_PASSWORD: &str = "peerfora-admin-password-for-integration";
pub(super) const TEST_PUBLIC_URL: &str = "https://peerfora.example.test";

pub(super) struct TestContext {
    pub database: PgPool,
    pub app: axum::Router,
    pub administrator_id: uuid::Uuid,
    pub settings: Settings,
    pub sent_emails: Arc<Mutex<Vec<String>>>,
}

pub(super) struct AdministratorAuthState {
    pub administrator_cookie: String,
    pub totp_secret: String,
    pub code: String,
    pub recovery_code: String,
}

pub(super) struct AccountCommunityFixture {
    pub student_account_id: uuid::Uuid,
    pub community_id: uuid::Uuid,
}

impl TestContext {
    pub async fn connect_from_environment() -> Option<Self> {
        let database_url = std::env::var("PEERFORA_TEST_DATABASE_URL").ok()?;
        let encryption_key = STANDARD.encode([9_u8; 32]);
        let database = connect_database(&database_url)
            .await
            .expect("the disposable PostgreSQL test database must migrate");
        assert_seeded_root_administrator_is_pending(&database).await;
        let administrator_id =
            provision_first_administrator(&database, ADMIN_EMAIL, ADMIN_PASSWORD.to_owned())
                .await
                .expect("first administrator should be provisioned once");
        assert_bootstrap_cannot_be_repeated(&database).await;
        let settings = test_settings(database_url, encryption_key);
        let sent_emails = Arc::new(Mutex::new(Vec::new()));
        let email_sender: Arc<dyn EmailSender> = Arc::new(RecordingEmailSender {
            sent_emails: sent_emails.clone(),
        });
        let app_state = peerfora_api::models::app_state::AppState::from_settings(
            database.clone(),
            &settings,
            Some(email_sender),
        );
        Some(Self {
            database,
            app: build_app(app_state),
            administrator_id,
            settings,
            sent_emails,
        })
    }
}

pub(super) async fn verify_readiness_after_database_shutdown(context: &TestContext) {
    context.database.close().await;
    let app_state = peerfora_api::models::app_state::AppState::from_settings(
        context.database.clone(),
        &context.settings,
        None,
    );
    let app = build_app(app_state);
    let readiness = send_request(app.clone(), "GET", "/health/ready", None, None).await;
    assert_eq!(readiness.status(), StatusCode::SERVICE_UNAVAILABLE);
    let liveness = send_request(app, "GET", "/health", None, None).await;
    assert_eq!(liveness.status(), StatusCode::OK);
}

fn test_settings(database_url: String, encryption_key: String) -> Settings {
    Settings {
        host: "127.0.0.1".parse().expect("loopback address"),
        port: 0,
        log_filter: "warn".to_owned(),
        database_url,
        cookie_secure: true,
        production_environment: false,
        admin_mfa_encryption_key: Some(encryption_key),
        authentication_rate_limit_hash_key: "test-auth-rate-limit-key-must-be-long-enough"
            .to_owned(),
        allowed_origins: vec![
            TEST_PUBLIC_URL
                .parse()
                .expect("test origin should be a valid header"),
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
    }
}

pub(super) async fn assert_seeded_root_administrator_is_pending(database: &PgPool) {
    let administrators: i64 = sqlx::query_scalar("SELECT count(*) FROM administrators")
        .fetch_one(database)
        .await
        .expect("admin identity table should exist after migration");
    assert_eq!(administrators, 1, "use a clean disposable database");
    let root_state: (String, bool) =
        sqlx::query_as("SELECT status, is_seeded_root FROM administrators")
            .fetch_one(database)
            .await
            .expect("the migration-seeded root administrator should exist");
    assert_eq!(root_state, ("pending_activation".to_owned(), true));
}

pub(super) async fn create_analytics_viewer_test_session(
    database: &PgPool,
    granted_by_administrator_id: uuid::Uuid,
) -> String {
    let administrator_id = uuid::Uuid::new_v4();
    let session_id = uuid::Uuid::new_v4();
    let session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrators
            (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at)
        VALUES ($1, $2, 'test-only-invalid-hash', 'test-only-encrypted-secret', now())",
    )
    .bind(administrator_id)
    .bind(format!("viewer-{administrator_id}@example.test"))
    .execute(database)
    .await
    .expect("analytics test administrator should be created");
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
        VALUES ($1, $2, 'analytics_viewer', $3)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(administrator_id)
    .bind(granted_by_administrator_id)
    .execute(database)
    .await
    .expect("analytics viewer role should be assigned");
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &session_token,
    ))
    .execute(database)
    .await
    .expect("analytics viewer session should be created");
    session_token
}

pub(super) async fn assert_bootstrap_cannot_be_repeated(database: &PgPool) {
    let repeated_bootstrap = provision_first_administrator(
        database,
        "second-admin@example.test",
        "second-admin-password-long-enough".to_owned(),
    )
    .await;
    assert!(repeated_bootstrap.is_err());
}

pub(super) async fn assert_audit_records_are_append_only(database: &PgPool) {
    let result = sqlx::query(
        "UPDATE administrative_audit_logs SET reason = 'changed'
        WHERE id = (SELECT id FROM administrative_audit_logs LIMIT 1)",
    )
    .execute(database)
    .await;
    assert!(result.is_err());
}

pub(super) async fn send_json(
    app: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    send_request(
        app,
        "POST",
        path,
        None,
        Some(serde_json::to_string(&body).expect("JSON request body")),
    )
    .await
}

pub(super) async fn send_request(
    app: axum::Router,
    method: &str,
    path: &str,
    cookie: Option<&str>,
    body: Option<String>,
) -> axum::response::Response {
    send_request_with_origin(app, method, path, cookie, Some(TEST_PUBLIC_URL), body).await
}

pub(super) async fn send_request_with_origin(
    app: axum::Router,
    method: &str,
    path: &str,
    cookie: Option<&str>,
    origin: Option<&str>,
    body: Option<String>,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(origin) = origin {
        request = request.header(header::ORIGIN, origin);
    }
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    let body = match body {
        Some(body) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(body)
        }
        None => Body::empty(),
    };
    app.oneshot(request.body(body).expect("HTTP request"))
        .await
        .expect("router response")
}

pub(super) async fn send_cors_preflight_request(
    app: axum::Router,
    path: &str,
    origin: &str,
    requested_method: &str,
    requested_headers: &str,
) -> axum::response::Response {
    let request = Request::builder()
        .method("OPTIONS")
        .uri(path)
        .header(header::ORIGIN, origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, requested_method)
        .header(header::ACCESS_CONTROL_REQUEST_HEADERS, requested_headers)
        .body(Body::empty())
        .expect("CORS preflight request");
    app.oneshot(request).await.expect("CORS preflight response")
}

pub(super) async fn send_request_with_bearer(
    app: axum::Router,
    method: &str,
    path: &str,
    token: &str,
) -> axum::response::Response {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("HTTP request");
    app.oneshot(request).await.expect("router response")
}

pub(super) async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("HTTP response body");
    serde_json::from_slice(&body).expect("valid JSON response")
}

pub(super) fn cookie_value(response: &axum::response::Response) -> String {
    response
        .headers()
        .get(header::SET_COOKIE)
        .expect("admin session cookie")
        .to_str()
        .expect("valid cookie header")
        .split(';')
        .next()
        .and_then(|cookie| cookie.split_once('='))
        .map(|(_, value)| value.to_owned())
        .expect("cookie token")
}

pub(super) fn authenticator_code_from_uri(provisioning_uri: &str) -> String {
    let secret = provisioning_uri
        .split('?')
        .nth(1)
        .and_then(|query| {
            query.split('&').find_map(|parameter| {
                let (name, value) = parameter.split_once('=')?;
                (name == "secret").then_some(value)
            })
        })
        .expect("TOTP secret parameter");
    let secret = Secret::try_from_base32(secret).expect("base32 TOTP secret");
    Builder::new()
        .with_secret(secret.as_bytes())
        .build()
        .expect("TOTP configuration")
        .generate_current()
        .to_string()
}

struct RecordingEmailSender {
    sent_emails: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl EmailSender for RecordingEmailSender {
    async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        message: &str,
    ) -> Result<(), AccountAuthError> {
        let Ok(mut sent_emails) = self.sent_emails.lock() else {
            return Err(AccountAuthError::EmailDeliveryFailed);
        };
        sent_emails.push(format!("{recipient}\n{subject}\n{message}"));
        Ok(())
    }
}
