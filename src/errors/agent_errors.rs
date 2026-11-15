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
    }
}
