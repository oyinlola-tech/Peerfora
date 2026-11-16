use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents::{
        AGENT_CREDENTIAL_DEFAULT_LIFETIME_DAYS, AGENT_CREDENTIAL_MAX_LIFETIME_DAYS,
    },
    enums::agent::AgentCredentialEnvironment,
    enums::agent_error::AgentError,
    errors::app_error::AppError,
    models::agent_credential::AgentCredentialRecord,
    repositories::agent_credential_repository::{self, RotateAgentCredential},
    utils::{machine_credential, one_time_token},
};

pub struct RotatedAgentCredential {
    pub record: AgentCredentialRecord,
    pub secret: String,
}

pub async fn rotate_agent_credential(
    pool: &PgPool,
    operator_account_id: Uuid,
    agent_actor_id: Uuid,
    previous_credential_id: Uuid,
    expires_in_days: Option<u16>,
    environment: AgentCredentialEnvironment,
) -> Result<RotatedAgentCredential, AppError> {
    let expires_in_days = expires_in_days.unwrap_or(AGENT_CREDENTIAL_DEFAULT_LIFETIME_DAYS);
    if !(1..=AGENT_CREDENTIAL_MAX_LIFETIME_DAYS).contains(&expires_in_days) {
        return Err(AgentError::CredentialLifetimeInvalid.into());
    }
    let secret = machine_credential::create_agent_machine_credential();
    let record = agent_credential_repository::rotate_agent_credential(
        pool,
        RotateAgentCredential {
            operator_account_id,
            agent_actor_id,
            previous_credential_id,
            new_credential_id: Uuid::new_v4(),
            new_token_hash: one_time_token::hash_one_time_token(&secret),
            expires_in_days,
            environment,
        },
    )
    .await?;
    Ok(RotatedAgentCredential { record, secret })
}
