use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMIN_RECOVERY_CODE_COUNT,
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    repositories::admin_auth_repository,
    utils::{
        administrator_totp::build_administrator_totp,
        encrypted_secret::decrypt_secret,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
};

pub async fn regenerate_administrator_recovery_codes(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
    session_id: Uuid,
    code: &str,
    encryption_key: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<Vec<String>, AppError> {
    let encryption_key = encryption_key.ok_or(AdminAuthError::MfaEncryptionKeyUnavailable)?;
    let administrator =
        admin_auth_repository::find_administrator_mfa_secret(pool, administrator_id)
            .await?
            .ok_or(AdminAuthError::InvalidSession)?;
    let authenticator_secret = decrypt_secret(&administrator.encrypted_secret, encryption_key)?;
    let authenticator = build_administrator_totp(authenticator_secret, &administrator.email)?;
    let Some(matched_step) = authenticator.check_current(code) else {
        admin_auth_repository::record_failed_administrator_mfa_for_identity(
            pool,
            administrator_id,
            correlation_id,
        )
        .await?;
        return Err(AdminAuthError::InvalidMfaCode.into());
    };
    let recovery_codes = (0..ADMIN_RECOVERY_CODE_COUNT)
        .map(|_| create_one_time_token())
        .collect::<Vec<_>>();
    let recovery_code_hashes = recovery_codes
        .iter()
        .map(|recovery_code| hash_one_time_token(recovery_code))
        .collect::<Vec<_>>();
    admin_auth_repository::regenerate_administrator_recovery_codes(
        pool,
        administrator_id,
        session_id,
        matched_step,
        &recovery_code_hashes,
        correlation_id,
    )
    .await?;
    Ok(recovery_codes)
}
