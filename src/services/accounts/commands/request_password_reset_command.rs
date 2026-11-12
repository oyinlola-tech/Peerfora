use sqlx::PgPool;

use crate::{
    errors::app_error::AppError,
    repositories::account_repository,
    utils::one_time_token::{create_one_time_token, hash_one_time_token},
};

pub async fn request_password_reset(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(String, String)>, AppError> {
    let Some(account) =
        account_repository::find_account_by_email(pool, &email.trim().to_ascii_lowercase()).await?
    else {
        return Ok(None);
    };
    let reset_token = create_one_time_token();
    account_repository::create_password_reset_token(
        pool,
        account.id,
        &hash_one_time_token(&reset_token),
    )
    .await?;
    Ok(Some((account.email, reset_token)))
}
