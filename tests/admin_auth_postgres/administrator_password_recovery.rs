use super::support::*;
use axum::http::StatusCode;

const RESET_PASSWORD: &str = "peerfora-admin-password-after-recovery";

pub async fn verify_administrator_password_recovery(context: &TestContext) {
    let request_route =
        peerfora_api::constants::admin_auth::ADMIN_AUTH_PASSWORD_RESET_REQUEST_ROUTE;
    let confirm_route =
        peerfora_api::constants::admin_auth::ADMIN_AUTH_PASSWORD_RESET_CONFIRM_ROUTE;
    let known_response = request_reset(context, request_route, ADMIN_EMAIL).await;
    let unknown_response = request_reset(context, request_route, "unknown@example.test").await;
    assert_eq!(known_response.0, StatusCode::ACCEPTED);
    assert_eq!(unknown_response.0, StatusCode::ACCEPTED);
    assert_eq!(known_response.1, unknown_response.1);
    assert_eq!(known_response.1["message"], unknown_response.1["message"]);

    let first_token = latest_reset_token(context).await;
    let replaced_request = request_reset(context, request_route, ADMIN_EMAIL).await;
    assert_eq!(replaced_request.0, StatusCode::ACCEPTED);
    let reset_token = latest_reset_token(context).await;
    assert_ne!(first_token, reset_token);
    assert_reset_token_is_rejected(context, confirm_route, &first_token).await;

    let (session_id, challenge_token) = create_reset_revocation_fixtures(context).await;
    let reset_response = send_json(
        context.app.clone(),
        confirm_route,
        serde_json::json!({ "token": reset_token, "newPassword": RESET_PASSWORD }),
    )
    .await;
    assert_eq!(reset_response.status(), StatusCode::NO_CONTENT);
    assert!(reset_response.headers().get("cache-control").is_some());
    assert!(reset_response.headers().get("referrer-policy").is_some());
    let revoked_session: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM administrator_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&context.database)
            .await
            .expect("reset session should remain recorded");
    assert!(revoked_session.is_some());
    let challenge_consumed: bool = sqlx::query_scalar(
        "SELECT consumed_at IS NOT NULL FROM administrator_mfa_challenges
        WHERE token_hash = $1",
    )
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &challenge_token,
    ))
    .fetch_one(&context.database)
    .await
    .expect("reset MFA challenge should remain recorded");
    assert!(challenge_consumed);
    assert_reset_token_is_rejected(context, confirm_route, &reset_token).await;
    assert_reset_was_audited(context).await;

    let login = send_json(
        context.app.clone(),
        peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGIN_ROUTE,
        serde_json::json!({ "email": ADMIN_EMAIL, "password": RESET_PASSWORD }),
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);
    let login_result = response_json(login).await;
    assert_eq!(login_result["nextStep"], "verify_mfa");
    assert!(login_result.get("sessionToken").is_none());
}

async fn request_reset(
    context: &TestContext,
    route: &str,
    email: &str,
) -> (StatusCode, serde_json::Value) {
    let response = send_json(
        context.app.clone(),
        route,
        serde_json::json!({ "email": email }),
    )
    .await;
    let status = response.status();
    let body = response_json(response).await;
    (status, body)
}

async fn latest_reset_token(context: &TestContext) -> String {
    let reset_path = peerfora_api::constants::admin_auth::ADMIN_PASSWORD_RESET_PAGE_PATH;
    let token_parameter = peerfora_api::constants::account_auth::ONE_TIME_TOKEN_QUERY_PARAMETER;
    let reset_link_prefix = format!("{}{reset_path}?{token_parameter}=", TEST_PUBLIC_URL);
    let emails = context.sent_emails.lock().expect("recorded emails");
    let email = emails
        .iter()
        .rev()
        .find(|email| email.contains(ADMIN_EMAIL) && email.contains("/admin/reset-password"))
        .expect("administrator reset email");
    let link = email
        .lines()
        .find(|line| line.starts_with(&reset_link_prefix))
        .expect("reset link in email body");
    link.split_once(&format!("{token_parameter}="))
        .map(|(_, token)| token.to_owned())
        .expect("reset token query parameter")
}

async fn assert_reset_token_is_rejected(context: &TestContext, route: &str, token: &str) {
    let response = send_json(
        context.app.clone(),
        route,
        serde_json::json!({ "token": token, "newPassword": RESET_PASSWORD }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

async fn create_reset_revocation_fixtures(context: &TestContext) -> (uuid::Uuid, String) {
    let session_id = uuid::Uuid::new_v4();
    let session_token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(session_id)
    .bind(context.administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &session_token,
    ))
    .execute(&context.database)
    .await
    .expect("active session fixture should persist");

    let challenge_token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_mfa_challenges
            (token_hash, administrator_id, purpose, expires_at)
        VALUES ($1, $2, 'login', now() + interval '5 minutes')",
    )
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &challenge_token,
    ))
    .bind(context.administrator_id)
    .execute(&context.database)
    .await
    .expect("pending MFA fixture should persist");
    (session_id, challenge_token)
}

async fn assert_reset_was_audited(context: &TestContext) {
    let audited: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrative_audit_logs
            WHERE actor_administrator_id = $1
                AND action = 'administrator.password_reset_completed'
                AND result = 'success'
        )",
    )
    .bind(context.administrator_id)
    .fetch_one(&context.database)
    .await
    .expect("password reset audit query");
    assert!(audited);
}
