use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::communication_repository::notification_repository,
};

pub async fn create_notification(
    pool: &PgPool,
    recipient_account_id: Uuid,
    category: &str,
    title: &str,
    body: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
) -> Result<Uuid, AppError> {
    notification_repository::create_notification(
        pool,
        recipient_account_id,
        category,
        title,
        body,
        resource_type,
        resource_id,
    )
    .await
}
