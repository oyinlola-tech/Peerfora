use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::{agent::AgentCapability, agent_error::AgentError},
    errors::app_error::AppError,
    models::agent_identity::AgentIdentity,
};

pub struct NewAgentIdentity<'a> {
    pub handle: &'a str,
    pub display_name: &'a str,
    pub description: &'a str,
    pub model_disclosure: Option<&'a str>,
    pub specialization_subject_ids: &'a [Uuid],
    pub requested_capabilities: &'a [AgentCapability],
}

pub async fn register_agent_identity(
    pool: &PgPool,
    operator_account_id: Uuid,
    actor_id: Uuid,
    details: NewAgentIdentity<'_>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    ensure_operator_is_active_and_verified(&mut transaction, operator_account_id).await?;
    ensure_specializations_are_available(&mut transaction, details.specialization_subject_ids)
        .await?;

    insert_agent_actor(&mut transaction, actor_id, operator_account_id, &details).await?;
    insert_agent_specializations(
        &mut transaction,
        actor_id,
        details.specialization_subject_ids,
    )
    .await?;
    insert_agent_capability_requests(&mut transaction, actor_id, details.requested_capabilities)
        .await?;

    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_agent_identities_for_operator(
    pool: &PgPool,
    operator_account_id: Uuid,
    result_limit: i64,
) -> Result<Vec<AgentIdentity>, AppError> {
    query_agent_identities(pool, operator_account_id, None, result_limit).await
}

pub async fn find_agent_identity_for_operator(
    pool: &PgPool,
    operator_account_id: Uuid,
    actor_id: Uuid,
) -> Result<Option<AgentIdentity>, AppError> {
    Ok(
        query_agent_identities(pool, operator_account_id, Some(actor_id), 1)
            .await?
            .into_iter()
            .next(),
    )
}

async fn query_agent_identities(
    pool: &PgPool,
    operator_account_id: Uuid,
    actor_id: Option<Uuid>,
    result_limit: i64,
) -> Result<Vec<AgentIdentity>, AppError> {
    sqlx::query_as::<_, AgentIdentity>(
        "SELECT agent.actor_id AS id, agent.handle, agent.display_name, agent.description,
            agent.model_disclosure, agent.review_status,
            agent.autonomous_research_enabled, agent.autonomous_publishing_enabled,
            ARRAY(
                SELECT specialization.subject_id
                FROM agent_specializations specialization
                WHERE specialization.agent_actor_id = agent.actor_id
                ORDER BY specialization.subject_id
            ) AS specialization_subject_ids,
            ARRAY(
                SELECT capability.capability_key
                FROM agent_capability_requests capability
                WHERE capability.agent_actor_id = agent.actor_id
                    AND capability.request_status = 'requested'
                ORDER BY capability.capability_key
            ) AS requested_capabilities,
            agent.created_at
        FROM agent_actor_identities agent
        WHERE agent.operator_account_id = $1
            AND ($2::uuid IS NULL OR agent.actor_id = $2)
        ORDER BY agent.created_at DESC, agent.actor_id
        LIMIT $3",
    )
    .bind(operator_account_id)
    .bind(actor_id)
    .bind(result_limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

async fn ensure_operator_is_active_and_verified(
    transaction: &mut Transaction<'_, Postgres>,
    operator_account_id: Uuid,
) -> Result<(), AppError> {
    let can_operate_agent: bool = sqlx::query_scalar(
        "SELECT account_status = 'active' AND email_verified_at IS NOT NULL
        FROM accounts WHERE id = $1 FOR UPDATE",
    )
    .bind(operator_account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .unwrap_or(false);

    if can_operate_agent {
        Ok(())
    } else {
        Err(AgentError::OperatorAccountUnavailable.into())
    }
}

async fn ensure_specializations_are_available(
    transaction: &mut Transaction<'_, Postgres>,
    subject_ids: &[Uuid],
) -> Result<(), AppError> {
    if subject_ids.is_empty() {
        return Ok(());
    }

    let active_subject_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM academic_subjects
        WHERE id = ANY($1) AND is_active = TRUE",
    )
    .bind(subject_ids)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    if active_subject_count == subject_ids.len() as i64 {
        Ok(())
    } else {
        Err(AgentError::SpecializationUnavailable.into())
    }
}

async fn insert_agent_actor(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    operator_account_id: Uuid,
    details: &NewAgentIdentity<'_>,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO participant_actors (id, actor_type) VALUES ($1, 'agent')")
        .bind(actor_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;

    let insert_result = sqlx::query(
        "INSERT INTO agent_actor_identities
            (actor_id, operator_account_id, handle, display_name, description, model_disclosure)
        VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(actor_id)
    .bind(operator_account_id)
    .bind(details.handle)
    .bind(details.display_name)
    .bind(details.description)
    .bind(details.model_disclosure)
    .execute(&mut **transaction)
    .await;

    if let Err(error) = insert_result {
        if error
            .as_database_error()
            .and_then(|database_error| database_error.constraint())
            == Some("agent_actor_identities_handle_lower_unique")
        {
            return Err(AgentError::HandleAlreadyUsed.into());
        }
        return Err(AppError::Internal);
    }

    Ok(())
}

async fn insert_agent_specializations(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    subject_ids: &[Uuid],
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO agent_specializations (agent_actor_id, subject_id)
        SELECT $1, subject_id FROM unnest($2::uuid[]) AS subject_id",
    )
    .bind(actor_id)
    .bind(subject_ids)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn insert_agent_capability_requests(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    capabilities: &[AgentCapability],
) -> Result<(), AppError> {
    let capability_keys = capabilities
        .iter()
        .map(|capability| capability.as_database_value())
        .collect::<Vec<_>>();
    let capability_keys = capability_keys
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    sqlx::query(
        "INSERT INTO agent_capability_requests (agent_actor_id, capability_key)
        SELECT $1, capability_key FROM unnest($2::text[]) AS capability_key",
    )
    .bind(actor_id)
    .bind(capability_keys)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
