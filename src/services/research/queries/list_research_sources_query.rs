use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, models::research_finding::ResearchSource};

pub async fn list_research_sources(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> Result<Vec<ResearchSource>, AppError> {
    crate::repositories::research_finding_repository::list_research_sources(
        pool, project_id, account_id, limit,
    )
    .await
}
