use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::communication_repository::message_repository,
};

pub async fn mark_message_read(
    pool: &PgPool,
    account_id: Uuid,
    message_id: Uuid,
) -> Result<(), AppError> {
    message_repository::mark_message_read(pool, account_id, message_id).await
}

pub async fn report_message(
    pool: &PgPool,
    reporter_account_id: Uuid,
    message_id: Uuid,
    category: &str,
    description: &str,
) -> Result<(), AppError> {
    let parsed_category =
        crate::enums::moderation::ModerationReportCategory::from_category(category)?;
    crate::services::moderation::commands::create_moderation_report_command::create_moderation_report(
        pool,
        reporter_account_id,
        crate::models::moderation::ModerationReportTarget::Message(message_id),
        parsed_category,
        description,
    )
    .await?;
    Ok(())
}
