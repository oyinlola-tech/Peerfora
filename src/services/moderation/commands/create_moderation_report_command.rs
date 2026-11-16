use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::moderation::{
        MODERATION_REPORT_MAX_DESCRIPTION_LENGTH, MODERATION_REPORT_MIN_DESCRIPTION_LENGTH,
    },
    enums::moderation::{ModerationError, ModerationReportCategory},
    errors::app_error::AppError,
    models::moderation::ModerationReportTarget,
    repositories::moderation_repository,
};

pub async fn create_moderation_report(
    pool: &PgPool,
    reporter_account_id: Uuid,
    target: ModerationReportTarget,
    category: ModerationReportCategory,
    description: &str,
) -> Result<(Uuid, chrono::DateTime<chrono::Utc>), AppError> {
    let description = description.trim();
    let description_length = description.chars().count();
    if !(MODERATION_REPORT_MIN_DESCRIPTION_LENGTH..=MODERATION_REPORT_MAX_DESCRIPTION_LENGTH)
        .contains(&description_length)
    {
        return Err(ModerationError::DescriptionInvalid.into());
    }
    moderation_repository::create_report(pool, reporter_account_id, target, category, description)
        .await
}
