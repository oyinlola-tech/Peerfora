use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents::{AGENT_REVIEW_MAX_REASON_LENGTH, AGENT_REVIEW_MIN_REASON_LENGTH},
    enums::{agent::AgentLifecycleAction, agent_error::AgentError},
    errors::app_error::AppError,
    repositories::agent_administration_repository::lifecycle::administrate_agent_lifecycle,
};

pub async fn administrate_agent(
    pool: &PgPool,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    action: AgentLifecycleAction,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = validate_administrative_reason(reason)?;
    administrate_agent_lifecycle(
        pool,
        agent_actor_id,
        administrator_id,
        action,
        reason,
        correlation_id,
    )
    .await
}

pub(crate) fn validate_administrative_reason(reason: &str) -> Result<&str, AppError> {
    let reason = reason.trim();
    if !(AGENT_REVIEW_MIN_REASON_LENGTH..=AGENT_REVIEW_MAX_REASON_LENGTH)
        .contains(&reason.chars().count())
    {
        return Err(AgentError::AdministrativeReasonInvalid.into());
    }
    Ok(reason)
}
