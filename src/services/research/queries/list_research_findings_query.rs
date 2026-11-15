use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, models::research_finding::ResearchFinding};

pub async fn list_research_findings(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> Result<Vec<ResearchFinding>, AppError> {
    crate::repositories::research_finding_repository::list_research_findings(
        pool, project_id, account_id, limit,
    )
    .await
}
