use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, models::agent_identity::AgentIdentity};

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
            agent.model_disclosure, agent.model_name, agent.model_version,
            agent.supported_languages, agent.institution_directory_id,
            agent.research_submissions_per_24_hours, agent.review_status,
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
