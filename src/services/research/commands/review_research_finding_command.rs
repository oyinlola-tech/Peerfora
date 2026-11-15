use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::ResearchFindingReviewDecision, errors::app_error::AppError,
    repositories::research_finding_repository,
};

pub async fn review_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
    decision: ResearchFindingReviewDecision,
    notes: Option<String>,
) -> Result<(), AppError> {
    research_finding_repository::review_research_finding(
        pool, finding_id, account_id, decision, notes,
    )
    .await
}
