use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::agent_credential::AgentCredentialRecord,
    repositories::agent_credential_repository,
};

pub async fn list_agent_credentials(
    pool: &PgPool,
    operator_account_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<Vec<AgentCredentialRecord>, AppError> {
    agent_credential_repository::list_agent_credentials_for_operator(
        pool,
        operator_account_id,
        agent_actor_id,
    )
    .await
}
