use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::agent_credential::AgentResearchTask,
    repositories::research_task_repository,
};

pub async fn find_assigned_research_task_for_agent(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<AgentResearchTask, AppError> {
    research_task_repository::find_assigned_research_task_for_agent(pool, task_id, agent_actor_id)
        .await
}
