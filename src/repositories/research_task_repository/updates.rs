use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::{ResearchTaskError, ResearchTaskStatus},
    errors::app_error::AppError,
    models::research_task::ResearchTask,
};

use super::{access, queries};

pub struct ResearchTaskUpdate<'a> {
    pub title: &'a str,
    pub objective: &'a str,
    pub task_scope: &'a str,
    pub allowed_data_sources: &'a [String],
    pub permitted_tools: &'a [String],
    pub expected_deliverables: &'a [String],
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
}

pub async fn update_research_task(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
    details: ResearchTaskUpdate<'_>,
) -> Result<ResearchTask, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id =
        access::find_active_human_actor_for_account(&mut transaction, account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    let is_author = task.created_by_actor_id == actor_id;
    if is_author {
        access::ensure_active_project_member(&mut transaction, task.project_id, actor_id).await?;
    } else {
        access::ensure_project_task_manager(&mut transaction, task.project_id, actor_id).await?;
    }
    if task.task_status != ResearchTaskStatus::Draft.as_database_value() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }

    sqlx::query(
        "UPDATE research_tasks SET title = $2, objective = $3, task_scope = $4,
            allowed_data_sources = $5, permitted_tools = $6, expected_deliverables = $7,
            max_runtime_seconds = $8, max_result_bytes = $9, updated_at = now()
        WHERE id = $1",
    )
    .bind(task_id)
    .bind(details.title)
    .bind(details.objective)
    .bind(details.task_scope)
    .bind(details.allowed_data_sources)
    .bind(details.permitted_tools)
    .bind(details.expected_deliverables)
    .bind(details.max_runtime_seconds)
    .bind(details.max_result_bytes)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    transaction.commit().await.map_err(|_| AppError::Internal)?;
    queries::find_research_task_for_member(pool, task_id, account_id)
        .await?
        .ok_or(AppError::Internal)
}
