use sqlx::PgPool;

use crate::{
    constants::admin_auth::{ADMIN_PASSWORD_MAXIMUM_LENGTH, ADMIN_PASSWORD_MINIMUM_LENGTH},
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    repositories::admin_auth_repository,
    utils::{one_time_token::hash_one_time_token, password::create_password_hash},
};

pub async fn complete_administrator_password_reset(
    pool: &PgPool,
    token: &str,
    new_password: String,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    if !(ADMIN_PASSWORD_MINIMUM_LENGTH..=ADMIN_PASSWORD_MAXIMUM_LENGTH)
        .contains(&new_password.len())
    {
        return Err(AdminAuthError::InvalidAdministratorPasswordLength.into());
    }
    let password_hash = create_password_hash(new_password).await?;
    admin_auth_repository::complete_administrator_password_reset(
        pool,
        &hash_one_time_token(token),
        &password_hash,
        correlation_id,
    )
    .await
}
