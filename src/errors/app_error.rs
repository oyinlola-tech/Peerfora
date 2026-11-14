use crate::{
    enums::account_auth_error::AccountAuthError,
    enums::{community_error::CommunityError, directory_error::DirectoryError},
    errors::account_auth_errors::get_account_auth_error_response,
};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    Internal,
    AdminAuth(crate::enums::admin_auth_error::AdminAuthError),
    AccountAuth(AccountAuthError),
    Community(CommunityError),
    Directory(DirectoryError),
    Profile(crate::enums::profile_error::ProfileError),
    DatabaseConfigurationMissing,
    DatabaseUnavailable,
    DatabaseMigrationFailed,
    AuthenticationRateLimited,
    RequestOriginDenied,
    RequestTimedOut,
}

impl From<AccountAuthError> for AppError {
    fn from(error: AccountAuthError) -> Self {
        Self::AccountAuth(error)
    }
}

impl From<crate::enums::admin_auth_error::AdminAuthError> for AppError {
    fn from(error: crate::enums::admin_auth_error::AdminAuthError) -> Self {
        Self::AdminAuth(error)
    }
}

impl From<CommunityError> for AppError {
    fn from(error: CommunityError) -> Self {
        Self::Community(error)
    }
}

impl From<DirectoryError> for AppError {
    fn from(error: DirectoryError) -> Self {
        Self::Directory(error)
    }
}

impl From<crate::enums::profile_error::ProfileError> for AppError {
    fn from(error: crate::enums::profile_error::ProfileError) -> Self {
        Self::Profile(error)
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("application startup or request error")
    }
}

impl std::error::Error for AppError {}

#[derive(Serialize)]
struct ErrorResponse {
    message: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred",
            ),
            Self::AccountAuth(error) => get_account_auth_error_response(error),
            Self::AdminAuth(error) => {
                crate::errors::admin_auth_errors::get_admin_auth_error_response(error)
            }
            Self::Community(error) => {
                crate::errors::community_errors::get_community_error_response(error)
            }
            Self::Directory(error) => {
                crate::errors::directory_errors::get_directory_error_response(error)
            }
            Self::Profile(error) => {
                crate::errors::profile_errors::get_profile_error_response(error)
            }
            Self::DatabaseConfigurationMissing => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database configuration is missing",
            ),
            Self::DatabaseUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Database is unavailable")
            }
            Self::DatabaseMigrationFailed => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database setup is incomplete",
            ),
            Self::AuthenticationRateLimited => {
                crate::errors::authentication_rate_limit_errors::get_authentication_rate_limit_error_response()
            }
            Self::RequestOriginDenied => {
                crate::errors::request_origin_errors::get_request_origin_error_response()
            }
            Self::RequestTimedOut => {
                crate::errors::request_timeout_errors::get_request_timeout_error_response()
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}
