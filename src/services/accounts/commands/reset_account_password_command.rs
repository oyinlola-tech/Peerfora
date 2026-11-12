use sqlx::PgPool;

use crate::{
    constants::account_auth::{MAX_PASSWORD_LENGTH_BYTES, MIN_PASSWORD_LENGTH_BYTES},
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    repositories::account_repository,
    utils::{one_time_token::hash_one_time_token, password::create_password_hash},
};

pub async fn reset_account_password(
    pool: &PgPool,
    token: &str,
    new_password: String,
) -> Result<(), AppError> {
    if !(MIN_PASSWORD_LENGTH_BYTES..=MAX_PASSWORD_LENGTH_BYTES).contains(&new_password.len()) {
        return Err(AccountAuthError::PasswordLengthInvalid.into());
    }
    let password_hash = create_password_hash(new_password).await?;
    account_repository::reset_account_password(pool, &hash_one_time_token(token), &password_hash)
        .await
}
