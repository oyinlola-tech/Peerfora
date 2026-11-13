use std::{collections::HashMap, env};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use peerfora_api::{
    app::build_app, configs::settings::Settings, database::connect_database,
    services::admin_auth::commands::provision_first_administrator_command::provision_first_administrator,
};
use sqlx::PgPool;
use totp_rs::{Builder, Secret};
use tower::ServiceExt;

const ADMIN_EMAIL: &str = "phase-three@example.test";
const ADMIN_PASSWORD: &str = "peerfora-admin-password-for-integration";

#[tokio::test]
async fn administrator_login_mfa_session_and_audit_use_postgresql() {
    let Ok(database_url) = env::var("PEERFORA_TEST_DATABASE_URL") else {
        return;
    };
    let encryption_key = STANDARD.encode([9_u8; 32]);
    let database = connect_database(&database_url)
        .await
        .expect("the disposable PostgreSQL test database must migrate");
    assert_empty_administrator_database(&database).await;
    let administrator_id =
        provision_first_administrator(&database, ADMIN_EMAIL, ADMIN_PASSWORD.to_owned())
            .await
            .expect("first administrator should be provisioned once");
    assert_bootstrap_cannot_be_repeated(&database).await;

    let settings = test_settings(database_url, encryption_key);
    let app_state =
        peerfora_api::models::app_state::AppState::from_settings(database.clone(), &settings, None);
    let app = build_app(app_state);

    let student_cookie_response = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        Some("peerfora_session=student-session-value"),
        None,
    )
    .await;
    assert_eq!(student_cookie_response.status(), StatusCode::UNAUTHORIZED);

    let student_bearer_response = send_request_with_bearer(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        "student-session-value",
    )
    .await;
    assert_eq!(student_bearer_response.status(), StatusCode::UNAUTHORIZED);

    let login_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    assert_eq!(login_response.status(), StatusCode::OK);
    let login_json = response_json(login_response).await;
    assert_eq!(login_json["nextStep"], "enroll_mfa");
    let enrollment_token = login_json["challengeToken"]
        .as_str()
        .expect("MFA enrollment challenge token");

    let provisioning_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/enroll",
        serde_json::json!({ "challengeToken": enrollment_token }),
    )
    .await;
    assert_eq!(provisioning_response.status(), StatusCode::OK);
    let provisioning_json = response_json(provisioning_response).await;
    let provisioning_uri = provisioning_json["provisioningUri"]
        .as_str()
        .expect("authenticator provisioning URI");
    let code = authenticator_code_from_uri(provisioning_uri);

    let enrollment_confirmation = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/confirm",
        serde_json::json!({ "challengeToken": enrollment_token, "code": code }),
    )
    .await;
    assert_eq!(enrollment_confirmation.status(), StatusCode::OK);
    let administrator_cookie = cookie_value(&enrollment_confirmation);
    let enrolled_profile = response_json(enrollment_confirmation).await;
    assert_eq!(
        enrolled_profile["administratorId"],
        administrator_id.to_string()
    );
    assert_eq!(enrolled_profile["roles"][0], "super_administrator");
    assert!(
        enrolled_profile["permissions"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );

    let reused_enrollment_challenge = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/confirm",
        serde_json::json!({ "challengeToken": enrollment_token, "code": code }),
    )
    .await;
    assert_eq!(
        reused_enrollment_challenge.status(),
        StatusCode::UNAUTHORIZED
    );

    let current_profile = send_request(
        app.clone(),
        "GET",
        "/api/v1/admin/auth/me",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(current_profile.status(), StatusCode::OK);

    let second_login_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/login",
        serde_json::json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
    )
    .await;
    let second_login_json = response_json(second_login_response).await;
    assert_eq!(second_login_json["nextStep"], "verify_mfa");
    let replay_response = send_json(
        app.clone(),
        "/api/v1/admin/auth/mfa/verify",
        serde_json::json!({
            "challengeToken": second_login_json["challengeToken"],
            "code": code
        }),
    )
    .await;
    assert_eq!(replay_response.status(), StatusCode::UNAUTHORIZED);

    let logout_response = send_request(
        app.clone(),
        "POST",
        "/api/v1/admin/auth/logout",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);
    let revoked_session_response = send_request(
        app,
        "GET",
        "/api/v1/admin/auth/me",
        Some(&format!("peerfora_admin_session={administrator_cookie}")),
        None,
    )
    .await;
    assert_eq!(revoked_session_response.status(), StatusCode::UNAUTHORIZED);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrative_audit_logs WHERE actor_administrator_id = $1",
    )
    .bind(administrator_id)
    .fetch_one(&database)
    .await
    .expect("audit rows should persist");
    assert!(audit_count >= 3);
    assert_audit_records_are_append_only(&database).await;
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
        country_policies: HashMap::new(),
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        smtp_from: None,
        public_url: "https://peerfora.example.test".to_owned(),
        google_oidc: None,
        apple_oidc: None,
    }
}

async fn assert_empty_administrator_database(database: &PgPool) {
    let administrators: i64 = sqlx::query_scalar("SELECT count(*) FROM administrators")
        .fetch_one(database)
        .await
        .expect("admin identity table should exist after migration");
    assert_eq!(administrators, 0, "use a clean disposable database");
}

async fn assert_bootstrap_cannot_be_repeated(database: &PgPool) {
    let repeated_bootstrap = provision_first_administrator(
        database,
        "second-admin@example.test",
        "second-admin-password-long-enough".to_owned(),
    )
    .await;
    assert!(repeated_bootstrap.is_err());
}

async fn assert_audit_records_are_append_only(database: &PgPool) {
    let result = sqlx::query(
        "UPDATE administrative_audit_logs SET reason = 'changed'
        WHERE id = (SELECT id FROM administrative_audit_logs LIMIT 1)",
    )
    .execute(database)
    .await;
    assert!(result.is_err());
}

async fn send_json(
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

async fn send_request(
    app: axum::Router,
    method: &str,
    path: &str,
    cookie: Option<&str>,
    body: Option<String>,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(path);
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

async fn send_request_with_bearer(
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

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("HTTP response body");
    serde_json::from_slice(&body).expect("valid JSON response")
}

fn cookie_value(response: &axum::response::Response) -> String {
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

fn authenticator_code_from_uri(provisioning_uri: &str) -> String {
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
