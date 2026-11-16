use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_APPLICATION_MAX_REVIEW_REASON_LENGTH,
        DEVELOPER_APPLICATION_MIN_REVIEW_REASON_LENGTH,
    },
    enums::developer::{DeveloperApplicationReviewDecision, DeveloperError},
    errors::app_error::AppError,
    repositories::developer_application_repository,
};

pub async fn review_developer_application(
    pool: &PgPool,
    application_id: Uuid,
    administrator_id: Uuid,
    decision: DeveloperApplicationReviewDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = reason.trim();
    if !(DEVELOPER_APPLICATION_MIN_REVIEW_REASON_LENGTH
        ..=DEVELOPER_APPLICATION_MAX_REVIEW_REASON_LENGTH)
        .contains(&reason.chars().count())
    {
        return Err(DeveloperError::ReviewReasonInvalid.into());
    }
    developer_application_repository::review_developer_application(
        pool,
        application_id,
        administrator_id,
        decision,
        reason,
        correlation_id,
    )
    .await
}
