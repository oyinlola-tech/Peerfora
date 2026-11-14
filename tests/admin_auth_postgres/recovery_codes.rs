use super::support::*;
use axum::http::{StatusCode, header};
use totp_rs::{Builder, Secret};

pub async fn verify_recovery_code_regeneration(
    context: &TestContext,
    administrator: &AdministratorAuthState,
) -> (Vec<String>, String) {
    let current_code = current_code_for_secret(&administrator.totp_secret);
    sqlx::query(
        "UPDATE administrators SET last_totp_step = last_totp_step - 1
        WHERE id = $1 AND last_totp_step IS NOT NULL",
    )
    .bind(context.administrator_id)
    .execute(&context.database)
    .await
    .expect("previous authenticator step should be adjusted for the fresh fixture");
    let response = regenerate_recovery_codes(context, administrator, &current_code).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let response_json = response_json(response).await;
    let recovery_codes = response_json["recoveryCodes"]
        .as_array()
        .expect("regenerated recovery codes should be returned once")
        .iter()
        .map(|value| value.as_str().expect("recovery code text").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(recovery_codes.len(), 10);
    assert_previous_unused_recovery_code_was_revoked(
        &context.database,
        context.administrator_id,
        &administrator.recovery_code,
    )
    .await;
    assert_new_recovery_code_is_stored_only_as_a_hash(
        &context.database,
        context.administrator_id,
        &recovery_codes[0],
    )
    .await;
    let replay = regenerate_recovery_codes(context, administrator, &current_code).await;
    assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
    (recovery_codes, current_code)
}

async fn regenerate_recovery_codes(
    context: &TestContext,
    administrator: &AdministratorAuthState,
    code: &str,
) -> axum::response::Response {
    send_request(
        context.app.clone(),
        "POST",
        "/api/v1/admin/auth/mfa/recovery-codes/regenerate",
        Some(&format!(
            "peerfora_admin_session={}",
            administrator.administrator_cookie
        )),
        Some(serde_json::json!({"code": code}).to_string()),
    )
    .await
}

async fn assert_previous_unused_recovery_code_was_revoked(
    database: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
    old_recovery_code: &str,
) {
    let was_revoked: bool = sqlx::query_scalar(
        "SELECT revoked_at IS NOT NULL FROM administrator_recovery_codes
        WHERE administrator_id = $1 AND token_hash = $2",
    )
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        old_recovery_code,
    ))
    .fetch_one(database)
    .await
    .expect("previous recovery-code record should remain");
    assert!(was_revoked);
}

async fn assert_new_recovery_code_is_stored_only_as_a_hash(
    database: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
    recovery_code: &str,
) {
    let stored_hash: String = sqlx::query_scalar(
        "SELECT token_hash FROM administrator_recovery_codes
        WHERE administrator_id = $1 AND token_hash = $2
            AND used_at IS NULL AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .bind(peerfora_api::utils::one_time_token::hash_one_time_token(
        recovery_code,
    ))
    .fetch_one(database)
    .await
    .expect("new recovery code hash should persist");
    assert_ne!(stored_hash, recovery_code);
}

fn current_code_for_secret(secret_text: &str) -> String {
    let secret = Secret::try_from_base32(secret_text).expect("TOTP secret should be base32");
    Builder::new()
        .with_secret(secret.as_bytes())
        .build()
        .expect("TOTP configuration should be valid")
        .generate_current()
        .to_string()
}
