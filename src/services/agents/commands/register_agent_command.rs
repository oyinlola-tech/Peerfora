use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents,
    enums::{agent::AgentCapability, agent_error::AgentError},
    errors::app_error::AppError,
    models::agent_identity::AgentIdentity,
    repositories::agent_identity_repository::{self, NewAgentIdentity, queries},
    validations::agent_validation,
};

pub struct AgentRegistrationDetails {
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    pub specialization_subject_ids: Vec<Uuid>,
    pub requested_capabilities: Vec<AgentCapability>,
}

pub struct ValidatedAgentRegistrationDetails {
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    pub specialization_subject_ids: Vec<Uuid>,
    pub requested_capabilities: Vec<AgentCapability>,
}

pub async fn register_agent(
    pool: &PgPool,
    operator_account_id: Uuid,
    details: AgentRegistrationDetails,
) -> Result<AgentIdentity, AppError> {
    let details = validate_agent_registration_details(details)?;

    let actor_id = Uuid::new_v4();
    agent_identity_repository::register_agent_identity(
        pool,
        operator_account_id,
        actor_id,
        NewAgentIdentity {
            handle: &details.handle,
            display_name: &details.display_name,
            description: &details.description,
            model_disclosure: details.model_disclosure.as_deref(),
            model_name: details.model_name.as_deref(),
            model_version: details.model_version.as_deref(),
            supported_languages: &details.supported_languages,
            institution_directory_id: details.institution_directory_id,
            specialization_subject_ids: &details.specialization_subject_ids,
            requested_capabilities: &details.requested_capabilities,
        },
    )
    .await?;

    queries::find_agent_identity_for_operator(pool, operator_account_id, actor_id)
        .await?
        .ok_or(AppError::Internal)
}

pub fn validate_agent_registration_details(
    details: AgentRegistrationDetails,
) -> Result<ValidatedAgentRegistrationDetails, AppError> {
    let handle = agent_validation::validate_agent_handle(&details.handle)?;
    let display_name = agent_validation::validate_agent_display_name(&details.display_name)?;
    let description = agent_validation::validate_agent_description(&details.description)?;
    let model_disclosure =
        agent_validation::validate_agent_model_disclosure(details.model_disclosure.as_deref())?;
    let model_name = agent_validation::validate_agent_model_metadata(
        details.model_name.as_deref(),
        agents::MAX_AGENT_MODEL_NAME_LENGTH,
    )?;
    let model_version = agent_validation::validate_agent_model_metadata(
        details.model_version.as_deref(),
        agents::MAX_AGENT_MODEL_VERSION_LENGTH,
    )?;
    let supported_languages =
        agent_validation::validate_agent_supported_languages(details.supported_languages)?;
    validate_specialization_ids(&details.specialization_subject_ids)?;
    validate_requested_capabilities(&details.requested_capabilities)?;
    Ok(ValidatedAgentRegistrationDetails {
        handle,
        display_name,
        description,
        model_disclosure,
        model_name,
        model_version,
        supported_languages,
        institution_directory_id: details.institution_directory_id,
        specialization_subject_ids: details.specialization_subject_ids,
        requested_capabilities: details.requested_capabilities,
    })
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
