use axum::http::StatusCode;

use crate::enums::developer::DeveloperError;

pub fn get_developer_error_response(error: DeveloperError) -> (StatusCode, &'static str) {
    match error {
        DeveloperError::ProfileNameInvalid => (
            StatusCode::BAD_REQUEST,
            "Developer display name must contain between 3 and 80 characters",
        ),
        DeveloperError::ProfileDescriptionInvalid => (
            StatusCode::BAD_REQUEST,
            "Developer description cannot exceed 1000 characters",
        ),
        DeveloperError::ProfileAlreadyExists => (
            StatusCode::CONFLICT,
            "A developer profile already exists for this account",
        ),
        DeveloperError::ProfileUnavailable => (
            StatusCode::FORBIDDEN,
            "An active, email-verified account with a developer profile is required",
        ),
        DeveloperError::ApplicationNameInvalid => (
            StatusCode::BAD_REQUEST,
            "Application name must contain between 3 and 100 characters",
        ),
        DeveloperError::ApplicationDescriptionInvalid => (
            StatusCode::BAD_REQUEST,
            "Application description cannot exceed 2000 characters",
        ),
        DeveloperError::NoRequestedScopes => (
            StatusCode::BAD_REQUEST,
            "An application must request at least one available scope",
        ),
        DeveloperError::TooManyRequestedScopes => (
            StatusCode::BAD_REQUEST,
            "An application requests more scopes than the platform currently permits",
        ),
        DeveloperError::DuplicateRequestedScope => (
            StatusCode::BAD_REQUEST,
            "An application scope can only be requested once",
        ),
        DeveloperError::RequestedScopeUnavailable => (
            StatusCode::BAD_REQUEST,
            "One or more requested scopes are unavailable",
        ),
        DeveloperError::ApplicationAlreadyExists => (
            StatusCode::CONFLICT,
            "An application with this name already exists in your developer account",
        ),
        DeveloperError::ApplicationNotFound => {
            (StatusCode::NOT_FOUND, "Developer application was not found")
        }
        DeveloperError::ApplicationCannotBeSubmitted => (
            StatusCode::CONFLICT,
            "Only a draft application can be submitted for review",
        ),
        DeveloperError::ApplicationNotAwaitingReview => (
            StatusCode::CONFLICT,
            "This application is not awaiting review",
        ),
        DeveloperError::ReviewReasonInvalid => (
            StatusCode::BAD_REQUEST,
            "Review reason must contain between 1 and 1000 characters",
        ),
        DeveloperError::CredentialInvalid => {
            (StatusCode::UNAUTHORIZED, "Developer credential is invalid")
        }
        DeveloperError::CredentialNameInvalid => (
            StatusCode::BAD_REQUEST,
            "Credential name must contain between 3 and 80 characters",
        ),
        DeveloperError::CredentialLifetimeInvalid => (
            StatusCode::BAD_REQUEST,
            "Credential lifetime must be between 1 and 365 days",
        ),
        DeveloperError::ApplicationNotApproved => (
            StatusCode::CONFLICT,
            "Only an approved application can receive credentials",
        ),
        DeveloperError::ApplicationSuspended => (
            StatusCode::CONFLICT,
            "A suspended application cannot use or issue credentials",
        ),
        DeveloperError::ApplicationStatusConflict => (
            StatusCode::CONFLICT,
            "This application cannot transition to the requested status",
        ),
        DeveloperError::NoApprovedScopes => (
            StatusCode::CONFLICT,
            "This application has no currently approved scopes",
        ),
        DeveloperError::CredentialLimitReached => (
            StatusCode::CONFLICT,
            "This application has reached its active credential limit",
        ),
        DeveloperError::CredentialNotFound => {
            (StatusCode::NOT_FOUND, "Developer credential was not found")
        }
        DeveloperError::CredentialNotActive => (
            StatusCode::CONFLICT,
            "This credential is expired or revoked",
        ),
        DeveloperError::CredentialScopeDenied => (
            StatusCode::FORBIDDEN,
            "The developer credential does not have the required approved scope",
        ),
        DeveloperError::CredentialEnvironmentMismatch => (
            StatusCode::UNAUTHORIZED,
            "The developer credential is invalid for this environment",
        ),
        DeveloperError::AdministrativeReasonInvalid => (
            StatusCode::BAD_REQUEST,
            "Reason must contain between 1 and 1000 characters",
        ),
    }
}
