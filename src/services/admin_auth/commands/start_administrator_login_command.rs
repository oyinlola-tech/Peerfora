use crate::{
    constants::admin_auth::{
        ADMIN_CHALLENGE_LIFETIME_MINUTES, ADMIN_PASSWORD_MAXIMUM_LENGTH,
        ADMIN_PASSWORD_MINIMUM_LENGTH,
    },
    enums::{admin_auth::AdministratorMfaChallengePurpose, admin_auth_error::AdminAuthError},
    errors::app_error::AppError,
    repositories::admin_auth_repository,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
        password::{verify_password_against_dummy_hash, verify_password_hash},
    },
};

pub struct AdministratorLoginChallenge {
    pub challenge_token: String,
    pub administrator_id: uuid::Uuid,
    pub email: String,
    pub next_step: &'static str,
}

pub async fn start_administrator_login(
    pool: &sqlx::PgPool,
    email: &str,
    password: String,
) -> Result<AdministratorLoginChallenge, AppError> {
    if !(ADMIN_PASSWORD_MINIMUM_LENGTH..=ADMIN_PASSWORD_MAXIMUM_LENGTH).contains(&password.len()) {
        return Err(AdminAuthError::InvalidCredentials.into());
    }
    let administrator =
        admin_auth_repository::find_administrator_for_login(pool, &normalize_email_address(email))
            .await?;
    let Some(administrator) = administrator else {
        verify_password_against_dummy_hash(password).await?;
        return Err(AdminAuthError::InvalidCredentials.into());
    };
    let Some(password_hash) = administrator.password_hash else {
        verify_password_against_dummy_hash(password).await?;
        return Err(AdminAuthError::InvalidCredentials.into());
    };
    if verify_password_hash(password, password_hash).await.is_err() || !administrator.is_active {
        admin_auth_repository::record_failed_administrator_login(pool, administrator.id).await?;
        return Err(AdminAuthError::InvalidCredentials.into());
    }
    if administrator.mfa_locked {
        return Err(AdminAuthError::InvalidCredentials.into());
    }
    admin_auth_repository::reset_administrator_login_failures(pool, administrator.id).await?;
    let (purpose, next_step) = if administrator.mfa_secret_ciphertext.is_some() {
        (AdministratorMfaChallengePurpose::Login, "verify_mfa")
    } else {
        (AdministratorMfaChallengePurpose::Enrollment, "enroll_mfa")
    };
    let challenge_token = create_one_time_token();
    admin_auth_repository::create_administrator_mfa_challenge(
        pool,
        administrator.id,
        &hash_one_time_token(&challenge_token),
        purpose,
    )
    .await?;
    Ok(AdministratorLoginChallenge {
        challenge_token,
        administrator_id: administrator.id,
        email: administrator.email,
        next_step,
    })
}

pub fn challenge_lifetime_seconds() -> u32 {
    (ADMIN_CHALLENGE_LIFETIME_MINUTES * 60) as u32
}
