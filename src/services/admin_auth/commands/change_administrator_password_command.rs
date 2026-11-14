use crate::{
    constants::admin_auth::{ADMIN_PASSWORD_MAXIMUM_LENGTH, ADMIN_PASSWORD_MINIMUM_LENGTH},
    enums::{account_auth_error::AccountAuthError, admin_auth_error::AdminAuthError},
    errors::app_error::AppError,
    repositories::admin_auth_repository,
    utils::password::{create_password_hash, verify_password_hash},
};

pub async fn change_administrator_password(
    pool: &sqlx::PgPool,
    administrator_id: uuid::Uuid,
    session_id: uuid::Uuid,
    current_password: String,
    new_password: String,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    validate_new_administrator_password(&current_password, &new_password)?;
    let current_password_hash =
        admin_auth_repository::find_active_administrator_password_hash(pool, administrator_id)
            .await?;
    if let Err(error) = verify_password_hash(current_password, current_password_hash.clone()).await
    {
        if matches!(
            error,
            AppError::AccountAuth(AccountAuthError::InvalidCredentials)
        ) {
            admin_auth_repository::record_failed_administrator_password_change(
                pool,
                administrator_id,
                correlation_id,
            )
            .await?;
            return Err(AdminAuthError::InvalidCredentials.into());
        }
        return Err(error);
    }
    let new_password_hash = create_password_hash(new_password).await?;
    admin_auth_repository::change_administrator_password(
        pool,
        administrator_id,
        session_id,
        &current_password_hash,
        &new_password_hash,
        correlation_id,
    )
    .await
}

fn validate_new_administrator_password(
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    if !(ADMIN_PASSWORD_MINIMUM_LENGTH..=ADMIN_PASSWORD_MAXIMUM_LENGTH)
        .contains(&new_password.len())
    {
        return Err(AdminAuthError::InvalidAdministratorPasswordLength.into());
    }
    if current_password == new_password {
        return Err(AdminAuthError::AdministratorPasswordUnchanged.into());
    }
    Ok(())
}
