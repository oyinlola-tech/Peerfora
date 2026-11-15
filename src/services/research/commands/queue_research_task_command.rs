use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_task_repository};

pub async fn queue_research_task(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    research_task_repository::queue_research_task(pool, task_id, account_id).await
}
