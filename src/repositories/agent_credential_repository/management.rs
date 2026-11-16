use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::agents::AGENT_CREDENTIAL_MAX_ACTIVE_PER_AGENT,
    enums::agent::{AgentCapability, AgentCredentialEnvironment},
    enums::agent_error::AgentError,
    errors::app_error::AppError,
    models::agent_credential::AgentCredentialRecord,
};

pub struct IssueAgentCredential {
    pub operator_account_id: Uuid,
    pub agent_actor_id: Uuid,
    pub credential_id: Uuid,
    pub token_hash: String,
    pub name: String,
    pub environment: AgentCredentialEnvironment,
    pub expires_in_days: u16,
    pub capabilities: Vec<AgentCapability>,
}

pub struct RotateAgentCredential {
    pub operator_account_id: Uuid,
    pub agent_actor_id: Uuid,
    pub previous_credential_id: Uuid,
    pub new_credential_id: Uuid,
    pub new_token_hash: String,
    pub expires_in_days: u16,
    pub environment: AgentCredentialEnvironment,
}

pub async fn issue_agent_credential(
    pool: &PgPool,
    request: IssueAgentCredential,
) -> Result<AgentCredentialRecord, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_approved_agent(
        &mut transaction,
        request.operator_account_id,
        request.agent_actor_id,
    )
    .await?;
    let approved_capabilities =
        find_approved_capabilities(&mut transaction, request.agent_actor_id).await?;
    validate_credential_capabilities(&request.capabilities, &approved_capabilities)?;
    ensure_credential_capacity(&mut transaction, request.agent_actor_id).await?;
    insert_agent_credential(
        &mut transaction,
        AgentCredentialInsert {
            credential_id: request.credential_id,
            agent_actor_id: request.agent_actor_id,
            token_hash: &request.token_hash,
            name: &request.name,
            environment: request.environment,
            expires_in_days: request.expires_in_days,
            capabilities: &request.capabilities,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    find_agent_credential(pool, request.credential_id).await
}

pub async fn revoke_agent_credential(
    pool: &PgPool,
    operator_account_id: Uuid,
    agent_actor_id: Uuid,
    credential_id: Uuid,
) -> Result<(), AppError> {
    let updated = sqlx::query(
        "UPDATE agent_machine_credentials credential
         SET revoked_at = COALESCE(credential.revoked_at, now())
         FROM agent_actor_identities agent
         WHERE credential.id = $1 AND credential.agent_actor_id = $2
             AND agent.actor_id = credential.agent_actor_id
             AND agent.operator_account_id = $3",
    )
    .bind(credential_id)
    .bind(agent_actor_id)
    .bind(operator_account_id)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if updated.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AgentError::CredentialNotFound.into())
    }
}

pub async fn rotate_agent_credential(
    pool: &PgPool,
    request: RotateAgentCredential,
) -> Result<AgentCredentialRecord, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_approved_agent(
        &mut transaction,
        request.operator_account_id,
        request.agent_actor_id,
    )
    .await?;
    let (name, environment, capabilities) = find_active_credential_for_rotation(
        &mut transaction,
        request.agent_actor_id,
        request.previous_credential_id,
    )
    .await?;
    if environment != request.environment.as_database_value() {
        return Err(AgentError::CredentialEnvironmentMismatch.into());
    }
    let approved_capabilities =
        find_approved_capabilities(&mut transaction, request.agent_actor_id).await?;
    let retained_capabilities = capabilities
        .into_iter()
        .filter_map(|key| AgentCapability::from_database_value(&key))
        .filter(|capability| approved_capabilities.contains(capability))
        .collect::<Vec<_>>();
    if retained_capabilities.is_empty() {
        return Err(AgentError::CredentialCapabilitiesInvalid.into());
    }
    revoke_locked_credential(&mut transaction, request.previous_credential_id).await?;
    ensure_credential_capacity(&mut transaction, request.agent_actor_id).await?;
    insert_agent_credential(
        &mut transaction,
        AgentCredentialInsert {
            credential_id: request.new_credential_id,
            agent_actor_id: request.agent_actor_id,
            token_hash: &request.new_token_hash,
            name: &name,
            environment: request.environment,
            expires_in_days: request.expires_in_days,
            capabilities: &retained_capabilities,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    find_agent_credential(pool, request.new_credential_id).await
}

struct AgentCredentialInsert<'a> {
    credential_id: Uuid,
    agent_actor_id: Uuid,
    token_hash: &'a str,
    name: &'a str,
    environment: AgentCredentialEnvironment,
    expires_in_days: u16,
    capabilities: &'a [AgentCapability],
}

async fn lock_approved_agent(
    transaction: &mut Transaction<'_, Postgres>,
    operator_account_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    let review_status: Option<String> = sqlx::query_scalar(
        "SELECT agent.review_status
         FROM agent_actor_identities agent
         JOIN accounts account ON account.id = agent.operator_account_id
         WHERE agent.actor_id = $1 AND agent.operator_account_id = $2
             AND account.account_status = 'active'
             AND account.email_verified_at IS NOT NULL
         FOR UPDATE OF agent",
    )
    .bind(agent_actor_id)
    .bind(operator_account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    match review_status.as_deref() {
        None => Err(AgentError::AgentNotFound.into()),
        Some("approved" | "active") => Ok(()),
        Some(_) => Err(AgentError::AgentNotApproved.into()),
    }
}

async fn find_approved_capabilities(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<Vec<AgentCapability>, AppError> {
    let capability_keys: Vec<String> = sqlx::query_scalar(
        "SELECT capability_key FROM agent_capability_requests
         WHERE agent_actor_id = $1 AND request_status = 'approved'
         ORDER BY capability_key",
    )
    .bind(agent_actor_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    capability_keys
        .iter()
        .map(|key| AgentCapability::from_database_value(key).ok_or(AppError::Internal))
        .collect()
}

fn validate_credential_capabilities(
    requested: &[AgentCapability],
    approved: &[AgentCapability],
) -> Result<(), AppError> {
    let unique_capabilities = requested.iter().collect::<std::collections::HashSet<_>>();
    if requested.is_empty()
        || unique_capabilities.len() != requested.len()
        || requested
            .iter()
            .any(|capability| !approved.contains(capability))
    {
        return Err(AgentError::CredentialCapabilitiesInvalid.into());
    }
    Ok(())
}

async fn ensure_credential_capacity(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    let active_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_machine_credentials
         WHERE agent_actor_id = $1 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(agent_actor_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_count >= AGENT_CREDENTIAL_MAX_ACTIVE_PER_AGENT {
        return Err(AgentError::CredentialLimitReached.into());
    }
    Ok(())
}

async fn insert_agent_credential(
    transaction: &mut Transaction<'_, Postgres>,
    credential: AgentCredentialInsert<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO agent_machine_credentials
            (id, agent_actor_id, token_hash, name, environment, expires_at)
         VALUES ($1, $2, $3, $4, $5, now() + make_interval(days => $6::integer))",
    )
    .bind(credential.credential_id)
    .bind(credential.agent_actor_id)
    .bind(credential.token_hash)
    .bind(credential.name)
    .bind(credential.environment.as_database_value())
    .bind(i32::from(credential.expires_in_days))
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    for capability in credential.capabilities {
        sqlx::query(
            "INSERT INTO agent_machine_credential_capabilities
                (credential_id, agent_actor_id, capability_key)
             SELECT $1, $2, requested.capability_key
             FROM agent_capability_requests requested
             WHERE requested.agent_actor_id = $2
                 AND requested.capability_key = $3 AND requested.request_status = 'approved'",
        )
        .bind(credential.credential_id)
        .bind(credential.agent_actor_id)
        .bind(capability.as_database_value())
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}

async fn find_active_credential_for_rotation(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    credential_id: Uuid,
) -> Result<(String, String, Vec<String>), AppError> {
    let record = sqlx::query_as::<_, (String, String, Vec<String>)>(
        "SELECT credential.name, credential.environment,
            ARRAY(SELECT capability.capability_key
                  FROM agent_machine_credential_capabilities capability
                  WHERE capability.credential_id = credential.id
                  ORDER BY capability.capability_key)
         FROM agent_machine_credentials credential
         WHERE credential.id = $1 AND credential.agent_actor_id = $2
             AND credential.revoked_at IS NULL AND credential.expires_at > now()
         FOR UPDATE",
    )
    .bind(credential_id)
    .bind(agent_actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    record.ok_or(AgentError::CredentialNotActive.into())
}

async fn revoke_locked_credential(
    transaction: &mut Transaction<'_, Postgres>,
    credential_id: Uuid,
) -> Result<(), AppError> {
    let revoked = sqlx::query(
        "UPDATE agent_machine_credentials SET revoked_at = now()
         WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(credential_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if revoked.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AgentError::CredentialNotActive.into())
    }
}

async fn find_agent_credential(
    pool: &PgPool,
    credential_id: Uuid,
) -> Result<AgentCredentialRecord, AppError> {
    sqlx::query_as::<_, AgentCredentialRecord>(
        "SELECT credential.id, credential.agent_actor_id, credential.name,
            credential.environment, credential.created_at, credential.expires_at,
            credential.revoked_at, credential.last_used_at,
            ARRAY(SELECT capability.capability_key
                  FROM agent_machine_credential_capabilities capability
                  WHERE capability.credential_id = credential.id
                  ORDER BY capability.capability_key) AS capabilities
         FROM agent_machine_credentials credential WHERE credential.id = $1",
    )
    .bind(credential_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}
