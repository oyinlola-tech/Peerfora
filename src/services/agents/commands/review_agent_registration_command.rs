use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents::{AGENT_REVIEW_MAX_REASON_LENGTH, AGENT_REVIEW_MIN_REASON_LENGTH},
    enums::{agent::AgentReviewDecision, agent_error::AgentError},
    errors::app_error::AppError,
    repositories::agent_administration_repository,
};

pub async fn review_agent_registration(
    pool: &PgPool,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    decision: AgentReviewDecision,
    approved_capabilities: &[crate::enums::agent::AgentCapability],
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = reason.trim();
    if !(AGENT_REVIEW_MIN_REASON_LENGTH..=AGENT_REVIEW_MAX_REASON_LENGTH)
        .contains(&reason.chars().count())
    {
        return Err(AgentError::ReviewReasonInvalid.into());
    }
    let unique_approvals = approved_capabilities.iter().collect::<HashSet<_>>();
    if unique_approvals.len() != approved_capabilities.len()
        || (decision == AgentReviewDecision::Reject && !approved_capabilities.is_empty())
    {
        return Err(AgentError::ApprovedCapabilitiesInvalid.into());
    }
    let approved_capability_keys = approved_capabilities
        .iter()
        .map(|capability| capability.as_database_value().to_owned())
        .collect::<Vec<_>>();
    agent_administration_repository::review_agent_registration(
        pool,
        agent_actor_id,
        administrator_id,
        decision,
        &approved_capability_keys,
        reason,
        correlation_id,
    )
    .await
}
