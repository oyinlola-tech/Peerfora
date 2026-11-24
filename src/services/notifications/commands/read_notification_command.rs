use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::communication_repository::notification_repository,
};

pub async fn read_notification(
    pool: &PgPool,
    account_id: Uuid,
    notification_id: Uuid,
) -> Result<bool, AppError> {
    notification_repository::read_notification(pool, account_id, notification_id).await
}

pub async fn read_all_notifications(pool: &PgPool, account_id: Uuid) -> Result<i64, AppError> {
    notification_repository::read_all_notifications(pool, account_id).await
}
