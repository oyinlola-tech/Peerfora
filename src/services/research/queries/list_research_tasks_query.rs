use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::research_task::ResearchTask,
    repositories::research_task_repository,
};

pub async fn list_research_tasks_for_account(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    requested_limit: Option<i64>,
) -> Result<Vec<ResearchTask>, AppError> {
    research_task_repository::require_active_project_member(pool, project_id, account_id).await?;
    research_task_repository::list_research_tasks_for_member(
        pool,
        project_id,
        account_id,
        requested_limit,
    )
    .await
}
