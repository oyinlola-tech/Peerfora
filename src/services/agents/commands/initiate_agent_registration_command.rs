use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents,
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    integrations::email::agent_operator_claim_email::send_agent_operator_claim_email,
    interfaces::email_sender::EmailSender,
    models::agent_identity::AgentOperatorClaim,
    repositories::{
        agent_identity_repository::{self, NewAgentIdentity},
        authentication_rate_limit_repository::hash_authentication_identifier,
    },
    services::agents::commands::register_agent_command::{self, AgentRegistrationDetails},
    utils::{email_address::normalize_email_address, one_time_token},
    validations::account_auth_validation::validate_email_address,
};

pub struct InitiateAgentRegistration<'a> {
    pub pool: &'a PgPool,
    pub operator_email: &'a str,
    pub details: AgentRegistrationDetails,
    pub authentication_rate_limit_hash_key: &'a str,
    pub public_url: &'a str,
    pub email_sender: Option<&'a dyn EmailSender>,
}

pub async fn initiate_agent_registration(
    request: InitiateAgentRegistration<'_>,
) -> Result<AgentOperatorClaim, AppError> {
    validate_email_address(request.operator_email).map_err(|_| AccountAuthError::InvalidEmail)?;
    let email_sender = request
        .email_sender
        .ok_or(AccountAuthError::EmailDeliveryNotConfigured)?;
    let details = register_agent_command::validate_agent_registration_details(request.details)?;
    let operator_email = normalize_email_address(request.operator_email);
    let operator_email_hash =
        hash_authentication_identifier(&operator_email, request.authentication_rate_limit_hash_key)
            .map_err(|_| AppError::Internal)?;
    let token = one_time_token::create_one_time_token();
    let claim = agent_identity_repository::register_unclaimed_agent_identity(
        request.pool,
        Uuid::new_v4(),
        Uuid::new_v4(),
        &operator_email_hash,
        &one_time_token::hash_one_time_token(&token),
        agents::AGENT_OPERATOR_CLAIM_LIFETIME_MINUTES,
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
    if let Err(email_error) = send_agent_operator_claim_email(
        request.public_url,
        email_sender,
        &operator_email,
        &details.handle,
        &token,
    )
    .await
    {
        agent_identity_repository::revoke_unclaimed_agent_registration_after_email_failure(
            request.pool,
            claim.agent_actor_id,
        )
        .await?;
        return Err(email_error);
    }
    Ok(claim)
}
