use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::ResearchTaskError, errors::app_error::AppError,
    models::research_task::ResearchTask, repositories::research_task_repository,
    validations::research_validation,
};

pub async fn submit_research_task_result(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
    result: &str,
) -> Result<ResearchTask, AppError> {
    let task = research_task_repository::find_research_task_for_member(pool, task_id, account_id)
        .await?
        .ok_or(ResearchTaskError::TaskNotFound)?;
    let result = research_validation::validate_research_task_result(result, task.max_result_bytes)?;
    research_task_repository::submit_research_task_result(pool, task_id, account_id, &result).await
}
