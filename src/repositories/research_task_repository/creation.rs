use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::ResearchTaskStatus, errors::app_error::AppError,
    models::research_task::ResearchTask,
};

use super::{access, queries};

pub struct NewResearchTask<'a> {
    pub title: &'a str,
    pub objective: &'a str,
    pub task_scope: &'a str,
    pub allowed_data_sources: &'a [String],
    pub permitted_tools: &'a [String],
    pub expected_deliverables: &'a [String],
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
}

pub async fn create_research_task(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    task_id: Uuid,
    details: NewResearchTask<'_>,
) -> Result<ResearchTask, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id =
        access::find_active_human_actor_for_account(&mut transaction, account_id).await?;
    access::lock_active_research_project(&mut transaction, project_id).await?;
    access::ensure_project_task_manager(&mut transaction, project_id, actor_id).await?;

    sqlx::query(
        "INSERT INTO research_tasks (
            id, project_id, created_by_actor_id, title, objective, task_scope,
            allowed_data_sources, permitted_tools, expected_deliverables,
            max_runtime_seconds, max_result_bytes, task_status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(task_id)
    .bind(project_id)
    .bind(actor_id)
    .bind(details.title)
    .bind(details.objective)
    .bind(details.task_scope)
    .bind(details.allowed_data_sources)
    .bind(details.permitted_tools)
    .bind(details.expected_deliverables)
    .bind(details.max_runtime_seconds)
    .bind(details.max_result_bytes)
    .bind(ResearchTaskStatus::Draft.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    transaction.commit().await.map_err(|_| AppError::Internal)?;
    queries::find_research_task_for_member(pool, task_id, account_id)
        .await?
        .ok_or(AppError::Internal)
}
