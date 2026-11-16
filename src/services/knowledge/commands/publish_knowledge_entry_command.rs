use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    repositories::knowledge_repository::administration::publish_approved_knowledge_entry,
};

use super::review_knowledge_proposal_command::validate_review_reason;

pub async fn publish_knowledge_entry(
    pool: &PgPool,
    entry_id: Uuid,
    administrator_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = validate_review_reason(reason)?;
    publish_approved_knowledge_entry(pool, entry_id, administrator_id, reason, correlation_id).await
}
