use sqlx::PgPool;

use crate::{
    constants::moderation::{
        ADMIN_REPORT_DEFAULT_PAGE_SIZE, ADMIN_REPORT_MAX_OFFSET, ADMIN_REPORT_MAX_PAGE_SIZE,
    },
    enums::moderation::ModerationReportStatus,
    errors::app_error::AppError,
    models::moderation::ModerationReportPage,
    repositories::moderation_repository,
};

pub async fn list_moderation_reports(
    pool: &PgPool,
    status: Option<ModerationReportStatus>,
    requested_limit: Option<u32>,
    requested_offset: Option<u32>,
) -> Result<(Vec<ModerationReportPage>, u32, u32, i64), AppError> {
    let limit = requested_limit
        .unwrap_or(ADMIN_REPORT_DEFAULT_PAGE_SIZE)
        .clamp(1, ADMIN_REPORT_MAX_PAGE_SIZE);
    let offset = requested_offset.unwrap_or(0).min(ADMIN_REPORT_MAX_OFFSET);
    let (reports, total_count) =
        moderation_repository::list_reports(pool, status, limit, offset).await?;
    Ok((reports, limit, offset, total_count))
}
