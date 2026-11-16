use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents::{
        AGENT_CREDENTIAL_LAST_USED_UPDATE_MINUTES, AGENT_CREDENTIAL_MAX_REQUESTS_PER_MINUTE,
    },
    enums::agent::{AgentCapability, AgentCredentialEnvironment, AgentCredentialValidation},
    errors::app_error::AppError,
};

pub async fn validate_agent_credential_for_capability(
    pool: &PgPool,
    token_hash: &str,
    expected_environment: AgentCredentialEnvironment,
    required_capability: AgentCapability,
) -> Result<AgentCredentialValidation, AppError> {
    let credential: Option<(Uuid, Uuid, bool)> = sqlx::query_as(
        "SELECT credential.id, credential.agent_actor_id,
            EXISTS (
                SELECT 1 FROM agent_machine_credential_capabilities granted
                JOIN agent_capability_requests approved
                    ON approved.agent_actor_id = granted.agent_actor_id
                    AND approved.capability_key = granted.capability_key
                WHERE granted.credential_id = credential.id
                    AND granted.capability_key = $3
                    AND approved.request_status = 'approved'
            ) AS has_capability
         FROM agent_machine_credentials credential
         JOIN agent_actor_identities agent ON agent.actor_id = credential.agent_actor_id
         JOIN accounts operator ON operator.id = agent.operator_account_id
         WHERE credential.token_hash = $1
             AND credential.environment = $2
             AND credential.revoked_at IS NULL AND credential.expires_at > now()
             AND agent.review_status IN ('approved', 'active')
             AND operator.account_status = 'active'
             AND operator.email_verified_at IS NOT NULL",
    )
    .bind(token_hash)
    .bind(expected_environment.as_database_value())
    .bind(required_capability.as_database_value())
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some((credential_id, agent_actor_id, has_capability)) = credential else {
        return Ok(AgentCredentialValidation::Invalid);
    };
    if !has_capability {
        return Ok(AgentCredentialValidation::ScopeDenied);
    }
    let request_count = increment_credential_minute_count(pool, credential_id).await?;
    if request_count > AGENT_CREDENTIAL_MAX_REQUESTS_PER_MINUTE {
        return Ok(AgentCredentialValidation::RateLimited);
    }
    update_credential_last_used(pool, credential_id).await?;
    Ok(AgentCredentialValidation::Authorized(
        credential_id,
        agent_actor_id,
    ))
}

async fn increment_credential_minute_count(
    pool: &PgPool,
    credential_id: Uuid,
) -> Result<i32, AppError> {
    sqlx::query_scalar(
        "INSERT INTO agent_machine_credential_rate_limits
            (credential_id, window_started_at, request_count)
         VALUES ($1, now(), 1)
         ON CONFLICT (credential_id) DO UPDATE
         SET window_started_at = CASE
                 WHEN agent_machine_credential_rate_limits.window_started_at
                     < now() - interval '1 minute' THEN now()
                 ELSE agent_machine_credential_rate_limits.window_started_at END,
             request_count = CASE
                 WHEN agent_machine_credential_rate_limits.window_started_at
                     < now() - interval '1 minute' THEN 1
                 ELSE LEAST(agent_machine_credential_rate_limits.request_count + 1, $2) END
         RETURNING request_count",
    )
    .bind(credential_id)
    .bind(AGENT_CREDENTIAL_MAX_REQUESTS_PER_MINUTE + 1)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

async fn update_credential_last_used(pool: &PgPool, credential_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE agent_machine_credentials SET last_used_at = now()
         WHERE id = $1 AND (last_used_at IS NULL OR last_used_at <
             now() - make_interval(mins => $2::integer))",
    )
    .bind(credential_id)
    .bind(AGENT_CREDENTIAL_LAST_USED_UPDATE_MINUTES)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
