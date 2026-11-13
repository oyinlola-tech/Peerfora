use uuid::Uuid;

use crate::{
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    repositories::admin_auth_repository,
    utils::{
        administrator_totp::build_administrator_totp,
        encrypted_secret::decrypt_secret,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
};

pub struct CompletedAdministratorMfa {
    pub session_token: String,
    pub administrator_id: Uuid,
}

pub async fn confirm_administrator_mfa_enrollment(
    pool: &sqlx::PgPool,
    challenge_token: &str,
    code: &str,
    encryption_key: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<CompletedAdministratorMfa, AppError> {
    complete_administrator_mfa(
        pool,
        challenge_token,
        code,
        encryption_key,
        "enrollment",
        correlation_id,
    )
    .await
}

pub async fn verify_administrator_mfa_login(
    pool: &sqlx::PgPool,
    challenge_token: &str,
    code: &str,
    encryption_key: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<CompletedAdministratorMfa, AppError> {
    complete_administrator_mfa(
        pool,
        challenge_token,
        code,
        encryption_key,
        "login",
        correlation_id,
    )
    .await
}

async fn complete_administrator_mfa(
    pool: &sqlx::PgPool,
    challenge_token: &str,
    code: &str,
    encryption_key: Option<&str>,
    required_purpose: &str,
    correlation_id: Option<&str>,
) -> Result<CompletedAdministratorMfa, AppError> {
    let encryption_key = encryption_key.ok_or(AdminAuthError::MfaEncryptionKeyUnavailable)?;
    let challenge_hash = hash_one_time_token(challenge_token);
    let challenge =
        admin_auth_repository::find_active_administrator_challenge(pool, &challenge_hash)
            .await?
            .ok_or(AdminAuthError::InvalidChallenge)?;
    if !challenge.is_active || challenge.purpose != required_purpose {
        return Err(AdminAuthError::InvalidChallenge.into());
    }
    let encrypted_secret = if required_purpose == "enrollment" {
        challenge.encrypted_secret.clone()
    } else {
        let administrator =
            crate::repositories::admin_auth_repository::find_administrator_for_login(
                pool,
                &challenge.email,
            )
            .await?
            .ok_or(AdminAuthError::InvalidChallenge)?;
        administrator.mfa_secret_ciphertext
    }
    .ok_or(AdminAuthError::InvalidChallenge)?;
    let decrypted_secret = decrypt_secret(&encrypted_secret, encryption_key)?;
    let totp = build_administrator_totp(decrypted_secret, &challenge.email)?;
    let Some(matched_step) = totp.check_current(code) else {
        admin_auth_repository::record_failed_administrator_mfa_attempt(pool, &challenge_hash)
            .await?;
        return Err(AdminAuthError::InvalidMfaCode.into());
    };
    let enrolled_secret = (required_purpose == "enrollment").then_some(encrypted_secret.as_str());
    let session_token = create_one_time_token();
    let session_id = Uuid::new_v4();
    admin_auth_repository::complete_administrator_mfa_and_create_session(
        pool,
        &challenge_hash,
        matched_step,
        enrolled_secret,
        session_id,
        &hash_one_time_token(&session_token),
        correlation_id,
    )
    .await?;
    Ok(CompletedAdministratorMfa {
        session_token,
        administrator_id: challenge.administrator_id,
    })
}
