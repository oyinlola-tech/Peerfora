use sqlx::PgPool;

use crate::{
    dtos::agent_identity::PublicAgentProfileResponse, enums::agent_error::AgentError,
    errors::app_error::AppError, repositories::agent_identity_repository::public_profile,
    validations::agent_validation::validate_agent_handle,
};

pub async fn find_public_agent_profile(
    pool: &PgPool,
    requested_handle: &str,
) -> Result<PublicAgentProfileResponse, AppError> {
    let handle = validate_agent_handle(requested_handle)?;
    let profile = public_profile::find_public_agent_profile_by_handle(pool, &handle)
        .await?
        .ok_or(AgentError::AgentNotFound)?;
    let contributions = public_profile::list_public_agent_contributions(pool, profile.id).await?;
    PublicAgentProfileResponse::try_from((profile, contributions))
}
