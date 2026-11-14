use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_PASSWORD_MAXIMUM_LENGTH, ADMIN_PASSWORD_MINIMUM_LENGTH, SEEDED_ROOT_ADMINISTRATOR_ID,
    },
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    repositories::admin_auth_repository,
    utils::{email_address::normalize_email_address, password::create_password_hash},
    validations::account_auth_validation::validate_email_address,
};

pub async fn provision_first_administrator(
    pool: &sqlx::PgPool,
    email: &str,
    password: String,
) -> Result<Uuid, AppError> {
    validate_email_address(email).map_err(|_| AdminAuthError::InvalidBootstrapInput)?;
    if !(ADMIN_PASSWORD_MINIMUM_LENGTH..=ADMIN_PASSWORD_MAXIMUM_LENGTH).contains(&password.len()) {
        return Err(AdminAuthError::InvalidAdministratorPasswordLength.into());
    }
    let password_hash = create_password_hash(password).await?;
    admin_auth_repository::bootstrap_first_administrator(
        pool,
        &normalize_email_address(email),
        &password_hash,
    )
    .await?;
    Uuid::parse_str(SEEDED_ROOT_ADMINISTRATOR_ID).map_err(|_| AppError::Internal)
}
