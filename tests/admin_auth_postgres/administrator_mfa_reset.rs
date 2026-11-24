use super::support::*;
use axum::http::StatusCode;

const TARGET_ADMINISTRATOR_PASSWORD: &str = "temporary-mfa-reset-password";

#[derive(sqlx::FromRow)]
struct AdministratorMfaState {
    encrypted_secret: Option<String>,
    enabled_at: Option<chrono::DateTime<chrono::Utc>>,
    last_totp_step: Option<i64>,
    failed_attempts: i16,
    locked_until: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn verify_administrator_mfa_reset(context: &TestContext) {
    let target_id = create_target_administrator(context).await;
    let root_session_token = create_root_administrator_session(context).await;
    let route = peerfora_api::constants::admin_auth::ADMINISTRATOR_MFA_RESET_ROUTE
        .replace("{id}", &target_id.to_string());
    let administrator_cookie = format!(
        "{}={root_session_token}",
        peerfora_api::constants::admin_auth::ADMIN_SESSION_COOKIE_NAME
    );
    let reset_response = send_request(
        context.app.clone(),
        "POST",
        &route,
        Some(&administrator_cookie),
        Some(serde_json::json!({ "reason": "security_concern" }).to_string()),
    )
    .await;
    assert_eq!(reset_response.status(), StatusCode::NO_CONTENT);
    assert_target_mfa_was_cleared(context, target_id).await;
    assert_target_sessions_were_revoked(context, target_id).await;
    assert_target_challenges_and_codes_were_revoked(context, target_id).await;
    assert_mfa_reset_was_audited(context, target_id).await;
    assert_fresh_totp_enrollment_is_required(context, target_id).await;

    let self_reset_route = peerfora_api::constants::admin_auth::ADMINISTRATOR_MFA_RESET_ROUTE
        .replace("{id}", &context.administrator_id.to_string());
    let self_reset_response = send_request(
        context.app.clone(),
        "POST",
        &self_reset_route,
        Some(&administrator_cookie),
        Some(serde_json::json!({ "reason": "security_concern" }).to_string()),
    )
    .await;
    assert_eq!(self_reset_response.status(), StatusCode::CONFLICT);
}

async fn create_target_administrator(context: &TestContext) -> uuid::Uuid {
    let target_id = uuid::Uuid::new_v4();
    let password_hash = peerfora_api::utils::password::create_password_hash(
        TARGET_ADMINISTRATOR_PASSWORD.to_owned(),
    )
    .await
    .expect("target administrator password hash");
    sqlx::query(
        "INSERT INTO administrators
            (id, email, password_hash, mfa_secret_ciphertext, mfa_enabled_at,
             failed_mfa_attempts, mfa_locked_until)
        VALUES ($1, $2, $3, 'test-only-encrypted-secret',
            now(), 4, now() + interval '10 minutes')",
    )
    .bind(target_id)
    .bind(format!("mfa-reset-{target_id}@example.test"))
    .bind(password_hash)
    .execute(&context.database)
    .await
    .expect("target administrator fixture should persist");

    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(target_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        "target-session",
    ))
    .execute(&context.database)
    .await
    .expect("target administrator session fixture should persist");
    sqlx::query(
        "INSERT INTO administrator_mfa_challenges
            (token_hash, administrator_id, purpose, expires_at)
        VALUES ($1, $2, 'login', now() + interval '5 minutes')",
    )
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        "target-challenge",
    ))
    .bind(target_id)
    .execute(&context.database)
    .await
    .expect("target administrator challenge fixture should persist");
    sqlx::query(
        "INSERT INTO administrator_recovery_codes (id, administrator_id, token_hash)
        VALUES ($1, $2, $3)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(target_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        "target-code",
    ))
    .execute(&context.database)
    .await
    .expect("target recovery-code fixture should persist");
    target_id
}

async fn create_root_administrator_session(context: &TestContext) -> String {
    let token = peerfora_api::utils::one_time_token::create_one_time_token();
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + interval '8 hours')",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(context.administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        &token,
    ))
    .execute(&context.database)
    .await
    .expect("root administrator session fixture should persist");
    token
}

async fn assert_target_mfa_was_cleared(context: &TestContext, target_id: uuid::Uuid) {
    let state = sqlx::query_as::<_, AdministratorMfaState>(
        "SELECT mfa_secret_ciphertext AS encrypted_secret, mfa_enabled_at AS enabled_at,
                last_totp_step, failed_mfa_attempts AS failed_attempts,
                mfa_locked_until AS locked_until
        FROM administrators WHERE id = $1",
    )
    .bind(target_id)
    .fetch_one(&context.database)
    .await
    .expect("target administrator should remain after MFA reset");
    assert!(state.encrypted_secret.is_none());
    assert!(state.enabled_at.is_none());
    assert!(state.last_totp_step.is_none());
    assert_eq!(state.failed_attempts, 0);
    assert!(state.locked_until.is_none());
}

async fn assert_fresh_totp_enrollment_is_required(context: &TestContext, target_id: uuid::Uuid) {
    let login = send_json(
        context.app.clone(),
        peerfora_api::constants::admin_auth::ADMIN_AUTH_LOGIN_ROUTE,
        serde_json::json!({
            "email": format!("mfa-reset-{target_id}@example.test"),
            "password": TARGET_ADMINISTRATOR_PASSWORD
        }),
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);
    let login_result = response_json(login).await;
    assert_eq!(login_result["nextStep"], "enroll_mfa");
    assert!(login_result.get("sessionToken").is_none());
}

async fn assert_target_sessions_were_revoked(context: &TestContext, target_id: uuid::Uuid) {
    let active_sessions: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM administrator_sessions
        WHERE administrator_id = $1 AND revoked_at IS NULL",
    )
    .bind(target_id)
    .fetch_one(&context.database)
    .await
    .expect("target session revocation query");
    assert_eq!(active_sessions, 0);
}

async fn assert_target_challenges_and_codes_were_revoked(
    context: &TestContext,
    target_id: uuid::Uuid,
) {
    let challenge_consumed: bool = sqlx::query_scalar(
        "SELECT consumed_at IS NOT NULL FROM administrator_mfa_challenges
        WHERE administrator_id = $1",
    )
    .bind(target_id)
    .fetch_one(&context.database)
    .await
    .expect("target challenge revocation query");
    let recovery_code_revoked: bool = sqlx::query_scalar(
        "SELECT revoked_at IS NOT NULL FROM administrator_recovery_codes
        WHERE administrator_id = $1",
    )
    .bind(target_id)
    .fetch_one(&context.database)
    .await
    .expect("target recovery-code revocation query");
    assert!(challenge_consumed);
    assert!(recovery_code_revoked);
}

async fn assert_mfa_reset_was_audited(context: &TestContext, target_id: uuid::Uuid) {
    let audited: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrative_audit_logs
            WHERE actor_administrator_id = $1 AND target_id = $2
                AND action = 'administrator.mfa_reset'
                AND reason = 'security_concern' AND result = 'success'
        )",
    )
    .bind(context.administrator_id)
    .bind(target_id)
    .fetch_one(&context.database)
    .await
    .expect("MFA reset audit query");
    assert!(audited);
}
