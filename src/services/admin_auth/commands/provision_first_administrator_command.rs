use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMIN_PASSWORD_MINIMUM_LENGTH,
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
    if password.len() < ADMIN_PASSWORD_MINIMUM_LENGTH {
        return Err(AdminAuthError::PasswordTooShort.into());
    }
    let administrator_id = Uuid::new_v4();
    let password_hash = create_password_hash(password).await?;
    admin_auth_repository::bootstrap_first_administrator(
        pool,
        administrator_id,
        &normalize_email_address(email),
        &password_hash,
    )
    .await?;
    Ok(administrator_id)
}
