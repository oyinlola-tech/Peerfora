use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{
        NOTIFICATION_DEFAULT_PAGE_SIZE, NOTIFICATION_MAX_PAGE_OFFSET, NOTIFICATION_MAX_PAGE_SIZE,
    },
    errors::app_error::AppError,
    models::communication::NotificationPage,
    repositories::communication_repository::notification_repository,
};

pub async fn list_notifications(
    pool: &PgPool,
    account_id: Uuid,
    include_read: bool,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(Vec<NotificationPage>, i64), AppError> {
    let limit = limit
        .unwrap_or(NOTIFICATION_DEFAULT_PAGE_SIZE)
        .clamp(1, NOTIFICATION_MAX_PAGE_SIZE);
    let offset = offset.unwrap_or(0).clamp(0, NOTIFICATION_MAX_PAGE_OFFSET);
    notification_repository::list_notifications(pool, account_id, include_read, limit, offset).await
}

pub async fn unread_notification_count(pool: &PgPool, account_id: Uuid) -> Result<i64, AppError> {
    notification_repository::unread_notification_count(pool, account_id).await
}

pub async fn get_notification_preferences(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<crate::models::communication::NotificationPreferences, AppError> {
    notification_repository::get_preferences(pool, account_id).await
}
