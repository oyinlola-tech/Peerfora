use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::knowledge,
    enums::{knowledge::KnowledgeReviewDecision, knowledge_error::KnowledgeError},
    errors::app_error::AppError,
    repositories::knowledge_repository::administration::review_knowledge_proposal,
};

pub async fn review_proposal(
    pool: &PgPool,
    entry_id: Uuid,
    administrator_id: Uuid,
    decision: KnowledgeReviewDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = validate_review_reason(reason)?;
    review_knowledge_proposal(
        pool,
        entry_id,
        administrator_id,
        decision,
        reason,
        correlation_id,
    )
    .await
}

pub(crate) fn validate_review_reason(reason: &str) -> Result<&str, AppError> {
    let reason = reason.trim();
    if !(knowledge::KNOWLEDGE_REVIEW_MIN_REASON_LENGTH
        ..=knowledge::KNOWLEDGE_REVIEW_MAX_REASON_LENGTH)
        .contains(&reason.chars().count())
    {
        return Err(KnowledgeError::ReviewReasonInvalid.into());
    }
    Ok(reason)
}
