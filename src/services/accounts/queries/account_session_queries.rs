use sqlx::PgPool;

use crate::utils::one_time_token::hash_one_time_token;
use crate::{
    errors::app_error::AppError, models::account::Account, repositories::account_repository,
};

pub async fn find_account_for_session(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Account>, AppError> {
    let token_hash = hash_one_time_token(token);
    account_repository::find_account_by_session(pool, &token_hash).await
}
