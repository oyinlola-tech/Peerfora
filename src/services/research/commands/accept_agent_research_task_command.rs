use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_task_repository};

pub async fn accept_research_task_assignment_for_agent(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    research_task_repository::accept_assigned_research_task_for_agent(pool, task_id, agent_actor_id)
        .await
}
