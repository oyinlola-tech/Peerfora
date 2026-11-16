use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_task_repository};

pub async fn assign_research_task_to_agent(
    pool: &PgPool,
    task_id: Uuid,
    manager_account_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    research_task_repository::assign_research_task_to_agent(
        pool,
        task_id,
        manager_account_id,
        agent_actor_id,
    )
    .await
}
