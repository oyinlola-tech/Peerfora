use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, models::research_finding::ResearchFinding};

pub async fn find_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Option<Uuid>,
) -> Result<ResearchFinding, AppError> {
    crate::repositories::research_finding_repository::find_research_finding(
        pool, finding_id, account_id,
    )
    .await
}
