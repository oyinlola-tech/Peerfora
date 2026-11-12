use sqlx::PgPool;

use crate::{
    errors::app_error::AppError, repositories::account_repository,
    utils::one_time_token::hash_one_time_token,
};

pub async fn end_account_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    account_repository::delete_session(pool, &hash_one_time_token(token)).await
}
