use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_finding_repository};

pub async fn submit_research_finding_for_review(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    research_finding_repository::submit_research_finding_for_review(pool, finding_id, account_id)
        .await
}
