use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::research_finding::ResearchFindingRevision,
    repositories::research_finding_repository,
};

pub async fn list_research_finding_revisions(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<Vec<ResearchFindingRevision>, AppError> {
    research_finding_repository::list_research_finding_revisions(pool, finding_id, account_id).await
}
