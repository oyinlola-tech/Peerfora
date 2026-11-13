use axum::http::StatusCode;

use crate::enums::admin_auth_error::AdminAuthError;

pub fn get_admin_auth_error_response(error: AdminAuthError) -> (StatusCode, &'static str) {
    match error {
        AdminAuthError::InvalidCredentials => {
            (StatusCode::UNAUTHORIZED, "Invalid administrator credentials")
        }
        AdminAuthError::InvalidChallenge => {
            (StatusCode::UNAUTHORIZED, "Administrator challenge is invalid or expired")
        }
        AdminAuthError::InvalidMfaCode => {
            (StatusCode::UNAUTHORIZED, "Administrator verification failed")
        }
        AdminAuthError::InvalidSession => {
            (StatusCode::UNAUTHORIZED, "Administrator session is invalid")
        }
        AdminAuthError::MfaEncryptionKeyUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Administrator multifactor authentication is unavailable",
        ),
        AdminAuthError::InvalidBootstrapInput => {
            (StatusCode::BAD_REQUEST, "Administrator bootstrap input is invalid")
        }
        AdminAuthError::BootstrapAlreadyCompleted => (
            StatusCode::CONFLICT,
            "Administrator bootstrap has already been completed",
        ),
        AdminAuthError::PasswordTooShort => (
            StatusCode::BAD_REQUEST,
            "Administrator password must contain at least 16 characters",
        ),
    }
}
