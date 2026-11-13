use crate::{
    enums::{admin_auth::AdministratorMfaChallengePurpose, admin_auth_error::AdminAuthError},
    errors::app_error::AppError,
    repositories::admin_auth_repository,
    utils::{
        administrator_totp::create_administrator_totp, encrypted_secret::encrypt_secret,
        one_time_token::hash_one_time_token,
    },
};

pub async fn begin_administrator_mfa_enrollment(
    pool: &sqlx::PgPool,
    challenge_token: &str,
    encryption_key: Option<&str>,
) -> Result<String, AppError> {
    let encryption_key = encryption_key.ok_or(AdminAuthError::MfaEncryptionKeyUnavailable)?;
    let challenge_hash = hash_one_time_token(challenge_token);
    let challenge =
        admin_auth_repository::find_active_administrator_challenge(pool, &challenge_hash)
            .await?
            .ok_or(AdminAuthError::InvalidChallenge)?;
    if !challenge.is_active
        || AdministratorMfaChallengePurpose::try_from(challenge.purpose.as_str())?
            != AdministratorMfaChallengePurpose::Enrollment
    {
        return Err(AdminAuthError::InvalidChallenge.into());
    }
    if let Some(encrypted_secret) = challenge.encrypted_secret {
        let secret =
            crate::utils::encrypted_secret::decrypt_secret(&encrypted_secret, encryption_key)?;
        let totp =
            crate::utils::administrator_totp::build_administrator_totp(secret, &challenge.email)?;
        return totp.to_url().map_err(|_| AppError::Internal);
    }
    let (totp, provisioning_uri) = create_administrator_totp(&challenge.email)?;
    let encrypted_secret = encrypt_secret(totp.secret().as_bytes(), encryption_key)?;
    admin_auth_repository::store_pending_administrator_mfa_secret(
        pool,
        &challenge_hash,
        &encrypted_secret,
    )
    .await?;
    Ok(provisioning_uri)
}
