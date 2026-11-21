use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::communication::DevicePushTokenSummary,
    repositories::communication_repository::push_repository,
};

pub async fn list_device_tokens(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<DevicePushTokenSummary>, AppError> {
    push_repository::list_device_tokens(pool, account_id).await
}

pub async fn get_device_token(
    pool: &PgPool,
    account_id: Uuid,
    token_id: Uuid,
) -> Result<DevicePushTokenSummary, AppError> {
    push_repository::get_device_token(pool, account_id, token_id).await
}
