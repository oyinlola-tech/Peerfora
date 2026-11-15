use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_task_repository};

pub async fn accept_research_task_assignment(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    research_task_repository::accept_research_task_assignment(pool, task_id, account_id).await
}
