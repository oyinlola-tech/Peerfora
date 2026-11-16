use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, models::research_finding::ResearchFinding};

pub async fn find_public_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
) -> Result<Option<ResearchFinding>, AppError> {
    crate::repositories::research_finding_repository::find_public_research_finding(pool, finding_id)
        .await
}
