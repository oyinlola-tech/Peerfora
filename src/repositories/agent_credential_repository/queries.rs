use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::agent_error::AgentError, errors::app_error::AppError,
    models::agent_credential::AgentCredentialRecord,
};

pub async fn list_agent_credentials_for_operator(
    pool: &PgPool,
    operator_account_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<Vec<AgentCredentialRecord>, AppError> {
    let agent_is_owned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM agent_actor_identities
            WHERE actor_id = $1 AND operator_account_id = $2
        )",
    )
    .bind(agent_actor_id)
    .bind(operator_account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if !agent_is_owned {
        return Err(AgentError::AgentNotFound.into());
    }
    sqlx::query_as::<_, AgentCredentialRecord>(
        "SELECT credential.id, credential.agent_actor_id, credential.name,
            credential.environment, credential.created_at, credential.expires_at,
            credential.revoked_at, credential.last_used_at,
            ARRAY(SELECT capability.capability_key
                  FROM agent_machine_credential_capabilities capability
                  WHERE capability.credential_id = credential.id
                  ORDER BY capability.capability_key) AS capabilities
         FROM agent_machine_credentials credential
         JOIN agent_actor_identities agent ON agent.actor_id = credential.agent_actor_id
         WHERE agent.operator_account_id = $1 AND agent.actor_id = $2
         ORDER BY credential.created_at DESC, credential.id DESC",
    )
    .bind(operator_account_id)
    .bind(agent_actor_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
