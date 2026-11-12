use crate::{
    enums::account_auth_error::AccountAuthError,
    errors::account_auth_errors::get_account_auth_error_response,
};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    Internal,
    AccountAuth(AccountAuthError),
    DatabaseConfigurationMissing,
    DatabaseUnavailable,
    DatabaseMigrationFailed,
}

impl From<AccountAuthError> for AppError {
    fn from(error: AccountAuthError) -> Self {
        Self::AccountAuth(error)
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
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}
