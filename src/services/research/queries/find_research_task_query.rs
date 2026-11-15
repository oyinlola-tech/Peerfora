use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::ResearchTaskError, errors::app_error::AppError,
    models::research_task::ResearchTask, repositories::research_task_repository,
};

pub async fn find_research_task_for_account(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
) -> Result<ResearchTask, AppError> {
    research_task_repository::find_research_task_for_member(pool, task_id, account_id)
        .await?
        .ok_or(ResearchTaskError::TaskNotFound.into())
}
