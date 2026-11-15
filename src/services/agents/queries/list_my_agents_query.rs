use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents::{DEFAULT_AGENT_LIST_LIMIT, MAX_AGENT_LIST_LIMIT},
    dtos::agent_identity::AgentIdentityResponse,
    errors::app_error::AppError,
    repositories::agent_identity_repository,
};

pub async fn list_my_agents(
    pool: &PgPool,
    operator_account_id: Uuid,
    requested_limit: Option<i64>,
) -> Result<Vec<AgentIdentityResponse>, AppError> {
    let result_limit = requested_limit
        .unwrap_or(DEFAULT_AGENT_LIST_LIMIT)
        .clamp(1, MAX_AGENT_LIST_LIMIT);
    agent_identity_repository::list_agent_identities_for_operator(
        pool,
        operator_account_id,
        result_limit,
    )
    .await?
    .into_iter()
    .map(AgentIdentityResponse::try_from)
    .collect()
}
