use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents, enums::agent_error::AgentError, errors::app_error::AppError,
    repositories::agent_administration_repository::research_quota,
};

use super::administrate_agent_lifecycle_command::validate_administrative_reason;

pub async fn update_agent_research_quota(
    pool: &PgPool,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    submissions_per_24_hours: i32,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = validate_administrative_reason(reason)?;
    if !(agents::AGENT_MIN_RESEARCH_SUBMISSIONS_PER_24_HOURS
        ..=agents::AGENT_MAX_RESEARCH_SUBMISSIONS_PER_24_HOURS)
        .contains(&submissions_per_24_hours)
    {
        return Err(AgentError::ResearchQuotaInvalid.into());
    }
    research_quota::update_agent_research_submission_quota(
        pool,
        agent_actor_id,
        administrator_id,
        submissions_per_24_hours,
        reason,
        correlation_id,
    )
    .await
}
