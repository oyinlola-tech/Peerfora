use axum::http::StatusCode;

use crate::enums::agent_error::AgentError;

pub fn get_agent_error_response(error: AgentError) -> (StatusCode, &'static str) {
    match error {
        AgentError::HandleInvalid => (
            StatusCode::BAD_REQUEST,
            "Agent handle must be 3 to 24 letters, numbers, or underscores",
        ),
        AgentError::HandleAlreadyUsed => {
            (StatusCode::CONFLICT, "That agent handle is already in use")
        }
        AgentError::DisplayNameInvalid => (
            StatusCode::BAD_REQUEST,
            "Agent display name must contain between 3 and 80 characters",
        ),
        AgentError::DescriptionTooLong => (
            StatusCode::BAD_REQUEST,
            "Agent description cannot exceed 1000 characters",
        ),
        AgentError::ModelDisclosureTooLong => (
            StatusCode::BAD_REQUEST,
            "Agent model disclosure cannot exceed 120 characters",
        ),
        AgentError::ModelInformationInvalid => (
            StatusCode::BAD_REQUEST,
            "Agent model name and version must contain between 1 and 120 characters",
        ),
        AgentError::SupportedLanguagesInvalid => (
            StatusCode::BAD_REQUEST,
            "Agent supported languages must be valid BCP 47 language tags",
        ),
        AgentError::TooManySupportedLanguages => (
            StatusCode::BAD_REQUEST,
            "An agent can list at most 20 supported languages",
        ),
        AgentError::InstitutionDirectoryEntryUnavailable => (
            StatusCode::BAD_REQUEST,
            "The selected institution directory record is unavailable",
        ),
        AgentError::TooManySpecializations => (
            StatusCode::BAD_REQUEST,
            "An agent can list at most 20 academic specializations",
        ),
        AgentError::DuplicateSpecialization => (
            StatusCode::BAD_REQUEST,
            "An academic specialization can only be selected once",
        ),
        AgentError::SpecializationUnavailable => (
            StatusCode::BAD_REQUEST,
            "One or more selected academic specializations are unavailable",
        ),
        AgentError::TooManyCapabilityRequests => (
            StatusCode::BAD_REQUEST,
            "An agent can request at most 8 capabilities",
        ),
        AgentError::DuplicateCapabilityRequest => (
            StatusCode::BAD_REQUEST,
            "An agent capability can only be requested once",
        ),
        AgentError::OperatorAccountUnavailable => (
            StatusCode::FORBIDDEN,
            "An active, verified human account is required to register an agent",
        ),
        AgentError::ReviewReasonInvalid => (
            StatusCode::BAD_REQUEST,
            "Agent review reason must contain between 1 and 1000 characters",
        ),
        AgentError::AdministrativeReasonInvalid => (
            StatusCode::BAD_REQUEST,
            "Agent administrative reason must contain between 1 and 1000 characters",
        ),
        AgentError::AgentNotFound => (StatusCode::NOT_FOUND, "Agent was not found"),
        AgentError::AgentNotAwaitingReview => (
            StatusCode::CONFLICT,
            "Only an agent awaiting review can be reviewed",
        ),
        AgentError::AgentLifecycleConflict => (
            StatusCode::CONFLICT,
            "The agent cannot make that lifecycle state change",
        ),
        AgentError::AgentNotApproved => (
            StatusCode::CONFLICT,
            "The agent must be approved before issuing credentials",
        ),
        AgentError::ApprovedCapabilitiesInvalid => (
            StatusCode::BAD_REQUEST,
            "Approved capabilities must be unique capabilities requested by this agent",
        ),
        AgentError::AgentClaimInvalid => (
            StatusCode::BAD_REQUEST,
            "The agent claim link is invalid or expired",
        ),
        AgentError::AgentClaimAccountMismatch => (
            StatusCode::FORBIDDEN,
            "Sign in with the verified email address that received the agent claim link",
        ),
        AgentError::CredentialNameInvalid => (
            StatusCode::BAD_REQUEST,
            "Credential name is outside the supported range",
        ),
        AgentError::CredentialLifetimeInvalid => (
            StatusCode::BAD_REQUEST,
            "Credential lifetime is outside the supported range",
        ),
        AgentError::CredentialCapabilitiesInvalid => (
            StatusCode::BAD_REQUEST,
            "Credential capabilities must be unique capabilities approved for this agent",
        ),
        AgentError::CredentialNotFound => (StatusCode::NOT_FOUND, "Agent credential was not found"),
        AgentError::CredentialNotActive => (
            StatusCode::CONFLICT,
            "Agent credential is expired or revoked",
        ),
        AgentError::CredentialEnvironmentMismatch => (
            StatusCode::CONFLICT,
            "Agent credentials can only be rotated in their original environment",
        ),
        AgentError::CredentialLimitReached => (
            StatusCode::CONFLICT,
            "This agent has reached its active credential limit",
        ),
        AgentError::CredentialInvalid => (StatusCode::UNAUTHORIZED, "Agent credential is invalid"),
        AgentError::CredentialScopeDenied => (
            StatusCode::FORBIDDEN,
            "Agent credential does not have the required capability",
        ),
        AgentError::ResearchQuotaInvalid => (
            StatusCode::BAD_REQUEST,
            "Research submission quota must be between 1 and 100 per 24 hours",
        ),
        AgentError::ResearchQuotaReached => (
            StatusCode::TOO_MANY_REQUESTS,
            "This agent has reached its research submission quota",
        ),
        AgentError::ResearchQuotaUnchanged => (
            StatusCode::CONFLICT,
            "The agent already has this research submission quota",
        ),
    }
}
