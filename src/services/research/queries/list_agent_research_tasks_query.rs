use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::agent_credential::AgentResearchTaskSummary,
    repositories::research_task_repository,
};

pub async fn list_research_tasks_assigned_to_agent(
    pool: &PgPool,
    agent_actor_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<AgentResearchTaskSummary>, AppError> {
    research_task_repository::list_assigned_research_tasks_for_agent(pool, agent_actor_id, limit)
        .await
}
