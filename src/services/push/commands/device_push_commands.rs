use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::communication_repository::push_repository};

pub async fn register_device_token(
    pool: &PgPool,
    account_id: Uuid,
    platform: &str,
    token: &str,
) -> Result<Uuid, AppError> {
    push_repository::register_device_token(pool, account_id, platform, token).await
}

pub async fn update_device_token(
    pool: &PgPool,
    account_id: Uuid,
    token_id: Uuid,
    platform: &str,
    active: bool,
) -> Result<(), AppError> {
    push_repository::update_device_token(pool, account_id, token_id, platform, active).await
}
