use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::ResearchTaskReviewDecision, errors::app_error::AppError,
    repositories::research_task_repository, validations::research_validation,
};

pub async fn review_research_task_result(
    pool: &PgPool,
    task_id: Uuid,
    reviewer_account_id: Uuid,
    decision: ResearchTaskReviewDecision,
    notes: Option<&str>,
) -> Result<(), AppError> {
    let notes = research_validation::validate_research_task_review_notes(notes)?;
    research_task_repository::review_research_task_submission(
        pool,
        task_id,
        reviewer_account_id,
        decision,
        notes.as_deref(),
    )
    .await
}
