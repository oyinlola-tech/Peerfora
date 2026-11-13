use axum::http::StatusCode;

use crate::enums::profile_error::ProfileError;

pub fn get_profile_error_response(error: ProfileError) -> (StatusCode, &'static str) {
    match error {
        ProfileError::BiographyTooLong => (
            StatusCode::BAD_REQUEST,
            "Biography cannot exceed 500 characters",
        ),
        ProfileError::InvalidProfileVisibility => (
            StatusCode::BAD_REQUEST,
            "Profile visibility must be public or private",
        ),
        ProfileError::ProfileNotFound => (StatusCode::NOT_FOUND, "Profile not found"),
        ProfileError::DisplayNameTooLong => (
            StatusCode::BAD_REQUEST,
            "Display name cannot exceed 120 characters",
        ),
        ProfileError::InvalidProfileReference => (
            StatusCode::BAD_REQUEST,
            "Selected academic directory reference was not found",
        ),
        ProfileError::ProfileReferenceTooLong => (
            StatusCode::BAD_REQUEST,
            "Profile reference cannot exceed 2048 bytes",
        ),
        ProfileError::InvalidPreferenceCountryCode => (
            StatusCode::BAD_REQUEST,
            "Country code must contain two letters",
        ),
        ProfileError::InvalidLanguageTag => (
            StatusCode::BAD_REQUEST,
            "Language tag is invalid or too long",
        ),
    }
}
