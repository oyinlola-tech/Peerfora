use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::{admin_auth, agents},
    enums::agent_error::AgentError,
    errors::app_error::AppError,
};

use super::lifecycle::{ensure_agent_management_permission, lock_administrator_authorization};

pub async fn update_agent_research_submission_quota(
    pool: &PgPool,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    submissions_per_24_hours: i32,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_authorization(&mut transaction).await?;
    ensure_agent_management_permission(&mut transaction, administrator_id).await?;
    acquire_agent_quota_lock(&mut transaction, agent_actor_id).await?;
    let previous_quota = find_agent_research_quota(&mut transaction, agent_actor_id).await?;
    if previous_quota == submissions_per_24_hours {
        return Err(AgentError::ResearchQuotaUnchanged.into());
    }
    save_agent_research_quota(&mut transaction, agent_actor_id, submissions_per_24_hours).await?;
    record_agent_research_quota_change(
        &mut transaction,
        agent_actor_id,
        administrator_id,
        previous_quota,
        submissions_per_24_hours,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn acquire_agent_quota_lock(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")
        .bind(agent_actor_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn find_agent_research_quota(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<i32, AppError> {
    let quota = sqlx::query_scalar(
        "SELECT research_submissions_per_24_hours
        FROM agent_actor_identities WHERE actor_id = $1",
    )
    .bind(agent_actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    quota.ok_or(AgentError::AgentNotFound.into())
}

async fn save_agent_research_quota(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    submissions_per_24_hours: i32,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE agent_actor_identities SET research_submissions_per_24_hours = $2,
            updated_at = now() WHERE actor_id = $1",
    )
    .bind(agent_actor_id)
    .bind(submissions_per_24_hours)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn record_agent_research_quota_change(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    previous_quota: i32,
    submissions_per_24_hours: i32,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id,
             result, reason, correlation_id, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(agents::AGENT_RESEARCH_QUOTA_UPDATED_AUDIT_ACTION)
    .bind(agents::AGENT_LIFECYCLE_AUDIT_TARGET_TYPE)
    .bind(agent_actor_id)
    .bind(admin_auth::ADMIN_AUDIT_SUCCESS_RESULT)
    .bind(reason)
    .bind(correlation_id)
    .bind(serde_json::json!({
        "previousSubmissionsPer24Hours": previous_quota,
        "submissionsPer24Hours": submissions_per_24_hours,
    }))
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
