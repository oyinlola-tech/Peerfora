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
    Agent(crate::enums::agent_error::AgentError),
    Directory(DirectoryError),
    Profile(crate::enums::profile_error::ProfileError),
    Research(crate::enums::research::ResearchProjectError),
    ResearchTask(crate::enums::research::ResearchTaskError),
    ResearchFinding(crate::enums::research::ResearchFindingError),
    Discussion(crate::enums::discussion::DiscussionError),
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

impl From<crate::enums::agent_error::AgentError> for AppError {
    fn from(error: crate::enums::agent_error::AgentError) -> Self {
        Self::Agent(error)
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

impl From<crate::enums::research::ResearchProjectError> for AppError {
    fn from(error: crate::enums::research::ResearchProjectError) -> Self {
        Self::Research(error)
    }
}

impl From<crate::enums::research::ResearchTaskError> for AppError {
    fn from(error: crate::enums::research::ResearchTaskError) -> Self {
        Self::ResearchTask(error)
    }
}

impl From<crate::enums::research::ResearchFindingError> for AppError {
    fn from(error: crate::enums::research::ResearchFindingError) -> Self {
        Self::ResearchFinding(error)
    }
}

impl From<crate::enums::discussion::DiscussionError> for AppError {
    fn from(error: crate::enums::discussion::DiscussionError) -> Self {
        Self::Discussion(error)
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
            Self::Agent(error) => crate::errors::agent_errors::get_agent_error_response(error),
            Self::Directory(error) => {
                crate::errors::directory_errors::get_directory_error_response(error)
            }
            Self::Profile(error) => {
                crate::errors::profile_errors::get_profile_error_response(error)
            }
            Self::Research(error) => {
                crate::errors::research_errors::get_research_project_error_response(error)
            }
            Self::ResearchTask(error) => {
                crate::errors::research_errors::get_research_task_error_response(error)
            }
            Self::ResearchFinding(error) => {
                crate::errors::research_finding_errors::get_research_finding_error_response(error)
            }
            Self::Discussion(error) => {
                crate::errors::discussion_errors::get_discussion_error_response(error)
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
