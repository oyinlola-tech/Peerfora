use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::moderation::{
        MODERATION_REPORT_MAX_RESOLUTION_LENGTH, MODERATION_REPORT_MIN_RESOLUTION_LENGTH,
    },
    enums::moderation::{ModerationError, ModerationReportDecision},
    errors::app_error::AppError,
    repositories::moderation_repository,
};

pub async fn resolve_moderation_report(
    pool: &PgPool,
    report_id: Uuid,
    administrator_id: Uuid,
    decision: ModerationReportDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = reason.trim();
    let reason_length = reason.chars().count();
    if !(MODERATION_REPORT_MIN_RESOLUTION_LENGTH..=MODERATION_REPORT_MAX_RESOLUTION_LENGTH)
        .contains(&reason_length)
    {
        return Err(ModerationError::ResolutionReasonInvalid.into());
    }
    moderation_repository::resolve_report(
        pool,
        report_id,
        administrator_id,
        decision,
        reason,
        correlation_id,
    )
    .await
}
