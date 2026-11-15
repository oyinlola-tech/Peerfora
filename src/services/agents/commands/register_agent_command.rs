use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents,
    enums::{agent::AgentCapability, agent_error::AgentError},
    errors::app_error::AppError,
    models::agent_identity::AgentIdentity,
    repositories::agent_identity_repository::{self, NewAgentIdentity},
    validations::agent_validation,
};

pub struct AgentRegistrationDetails {
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub specialization_subject_ids: Vec<Uuid>,
    pub requested_capabilities: Vec<AgentCapability>,
}

pub async fn register_agent(
    pool: &PgPool,
    operator_account_id: Uuid,
    details: AgentRegistrationDetails,
) -> Result<AgentIdentity, AppError> {
    let handle = agent_validation::validate_agent_handle(&details.handle)?;
    let display_name = agent_validation::validate_agent_display_name(&details.display_name)?;
    let description = agent_validation::validate_agent_description(&details.description)?;
    let model_disclosure =
        agent_validation::validate_agent_model_disclosure(details.model_disclosure.as_deref())?;
    validate_specialization_ids(&details.specialization_subject_ids)?;
    validate_requested_capabilities(&details.requested_capabilities)?;

    let actor_id = Uuid::new_v4();
    agent_identity_repository::register_agent_identity(
        pool,
        operator_account_id,
        actor_id,
        NewAgentIdentity {
            handle: &handle,
            display_name: &display_name,
            description: &description,
            model_disclosure: model_disclosure.as_deref(),
            specialization_subject_ids: &details.specialization_subject_ids,
            requested_capabilities: &details.requested_capabilities,
        },
    )
    .await?;

    agent_identity_repository::find_agent_identity_for_operator(pool, operator_account_id, actor_id)
        .await?
        .ok_or(AppError::Internal)
}

fn validate_specialization_ids(subject_ids: &[Uuid]) -> Result<(), AppError> {
    if subject_ids.len() > agents::MAX_AGENT_SPECIALIZATIONS {
        return Err(AgentError::TooManySpecializations.into());
    }
    if subject_ids.iter().collect::<HashSet<_>>().len() != subject_ids.len() {
        return Err(AgentError::DuplicateSpecialization.into());
    }
    Ok(())
}

fn validate_requested_capabilities(capabilities: &[AgentCapability]) -> Result<(), AppError> {
    if capabilities.len() > agents::MAX_AGENT_CAPABILITY_REQUESTS {
        return Err(AgentError::TooManyCapabilityRequests.into());
    }
    if capabilities.iter().collect::<HashSet<_>>().len() != capabilities.len() {
        return Err(AgentError::DuplicateCapabilityRequest.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{validate_requested_capabilities, validate_specialization_ids};
    use crate::enums::agent::AgentCapability;

    #[test]
    fn registration_capabilities_must_be_unique() {
        let error = validate_requested_capabilities(&[
            AgentCapability::Research,
            AgentCapability::Research,
        ])
        .unwrap_err();
        assert!(matches!(
            error,
            crate::errors::app_error::AppError::Agent(_)
        ));
    }

    #[test]
    fn registration_specializations_must_be_unique() {
        let subject_id = Uuid::new_v4();
        assert!(validate_specialization_ids(&[subject_id, subject_id]).is_err());
    }
}
