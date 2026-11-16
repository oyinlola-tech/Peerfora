use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::agents,
    enums::{
        agent::{AgentCapabilityReviewStatus, AgentReviewDecision, AgentReviewStatus},
        agent_error::AgentError,
    },
    errors::app_error::AppError,
    models::agent_identity::AgentReviewRecord,
};

pub mod lifecycle;
pub mod research_quota;

pub async fn list_agents_for_administrative_review(
    pool: &PgPool,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AgentReviewRecord>, i64), AppError> {
    let total_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_actor_identities
         WHERE $1::text IS NULL OR review_status = $1",
    )
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let items = sqlx::query_as::<_, AgentReviewRecord>(
        "SELECT agent.actor_id AS id, agent.operator_account_id,
            account.username AS operator_username, agent.handle, agent.display_name,
            agent.description, agent.model_disclosure, agent.model_name, agent.model_version,
            agent.supported_languages, agent.institution_directory_id,
            agent.research_submissions_per_24_hours, agent.review_status,
            ARRAY(
                SELECT capability.capability_key
                FROM agent_capability_requests capability
                WHERE capability.agent_actor_id = agent.actor_id
                    AND capability.request_status = $4
                ORDER BY capability.capability_key
            ) AS requested_capabilities,
            agent.created_at
         FROM agent_actor_identities agent
         LEFT JOIN accounts account ON account.id = agent.operator_account_id
         WHERE $1::text IS NULL OR agent.review_status = $1
         ORDER BY agent.created_at, agent.actor_id
         LIMIT $2 OFFSET $3",
    )
    .bind(status)
    .bind(limit)
    .bind(offset)
    .bind(AgentCapabilityReviewStatus::Requested.as_database_value())
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((items, total_count))
}

pub async fn review_agent_registration(
    pool: &PgPool,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    decision: AgentReviewDecision,
    approved_capabilities: &[String],
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let agent = lock_agent_registration(&mut transaction, agent_actor_id).await?;
    validate_capability_approvals(&agent.requested_capabilities, approved_capabilities)?;
    update_capability_request_decisions(
        &mut transaction,
        agent_actor_id,
        approved_capabilities,
        administrator_id,
    )
    .await?;
    update_agent_review_status(&mut transaction, agent_actor_id, decision).await?;
    insert_agent_review_audit(
        &mut transaction,
        agent_actor_id,
        administrator_id,
        decision,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

#[derive(FromRow)]
struct LockedAgentRegistration {
    review_status: String,
    requested_capabilities: Vec<String>,
}

async fn lock_agent_registration(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<LockedAgentRegistration, AppError> {
    let record: Option<LockedAgentRegistration> = sqlx::query_as(
        "SELECT agent.review_status,
            ARRAY(
                SELECT capability.capability_key
                FROM agent_capability_requests capability
                WHERE capability.agent_actor_id = agent.actor_id
                    AND capability.request_status = $2
                ORDER BY capability.capability_key
            ) AS requested_capabilities
         FROM agent_actor_identities agent
         WHERE agent.actor_id = $1
         FOR UPDATE OF agent",
    )
    .bind(agent_actor_id)
    .bind(AgentCapabilityReviewStatus::Requested.as_database_value())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let record = record.ok_or(AgentError::AgentNotFound)?;
    let review_status =
        AgentReviewStatus::from_database_value(&record.review_status).ok_or(AppError::Internal)?;
    if review_status != AgentReviewStatus::PendingReview {
        return Err(AgentError::AgentNotAwaitingReview.into());
    }
    Ok(record)
}

fn validate_capability_approvals(
    requested_capabilities: &[String],
    approved_capabilities: &[String],
) -> Result<(), AppError> {
    let unique_approvals = approved_capabilities
        .iter()
        .collect::<std::collections::HashSet<_>>();
    if unique_approvals.len() != approved_capabilities.len()
        || approved_capabilities
            .iter()
            .any(|capability| !requested_capabilities.contains(capability))
    {
        return Err(AgentError::ApprovedCapabilitiesInvalid.into());
    }
    Ok(())
}

async fn update_capability_request_decisions(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    approved_capabilities: &[String],
    administrator_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE agent_capability_requests
         SET request_status = CASE
                 WHEN capability_key = ANY($2::text[]) THEN $3
                 ELSE $4
             END,
             reviewed_by_administrator_id = $5,
             reviewed_at = now()
         WHERE agent_actor_id = $1 AND request_status = $6",
    )
    .bind(agent_actor_id)
    .bind(approved_capabilities)
    .bind(AgentCapabilityReviewStatus::Approved.as_database_value())
    .bind(AgentCapabilityReviewStatus::Denied.as_database_value())
    .bind(administrator_id)
    .bind(AgentCapabilityReviewStatus::Requested.as_database_value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn update_agent_review_status(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    decision: AgentReviewDecision,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE agent_actor_identities SET review_status = $2
         WHERE actor_id = $1 AND review_status = $3",
    )
    .bind(agent_actor_id)
    .bind(decision.resulting_status().as_database_value())
    .bind(AgentReviewStatus::PendingReview.as_database_value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn insert_agent_review_audit(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    decision: AgentReviewDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let action = match decision {
        AgentReviewDecision::Approve => agents::AGENT_APPROVED_AUDIT_ACTION,
        AgentReviewDecision::Reject => agents::AGENT_REJECTED_AUDIT_ACTION,
    };
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id, result, reason, correlation_id)
         VALUES ($1, $2, $3, $4, $5, 'success', $6, $7)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(action)
    .bind(agents::AGENT_REVIEW_AUDIT_TARGET_TYPE)
    .bind(agent_actor_id)
    .bind(reason)
    .bind(correlation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
