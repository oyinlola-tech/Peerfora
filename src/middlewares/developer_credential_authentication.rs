use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::{
    constants::developers::{
        DEVELOPER_CREDENTIAL_AUTHORIZATION_PREFIX, DEVELOPER_CREDENTIAL_PREFIX,
        DEVELOPER_CREDENTIAL_SECRET_LENGTH,
    },
    enums::developer::{
        DeveloperCredentialEnvironment, DeveloperCredentialValidation, DeveloperError,
    },
    errors::app_error::AppError,
    models::app_state::AppState,
    repositories::developer_application_repository::credentials,
    utils::one_time_token::hash_one_time_token,
};

#[derive(Clone)]
pub struct DeveloperCredentialGuard {
    pub database: PgPool,
    pub production_environment: bool,
    pub required_scope: &'static str,
}

impl DeveloperCredentialGuard {
    pub fn new(
        database: PgPool,
        production_environment: bool,
        required_scope: &'static str,
    ) -> Self {
        Self {
            database,
            production_environment,
            required_scope,
        }
    }
}

pub async fn require_developer_credential(
    State(guard): State<DeveloperCredentialGuard>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) = read_developer_bearer_credential(&request) else {
        return AppError::from(DeveloperError::CredentialInvalid).into_response();
    };
    let validation = credentials::validate_developer_credential_for_scope(
        &guard.database,
        &hash_one_time_token(token),
        get_runtime_credential_environment(guard.production_environment),
        guard.required_scope,
    )
    .await;
    match validation {
        Ok(DeveloperCredentialValidation::Authorized(credential_id, application_id)) => {
            request.extensions_mut().insert(
                crate::models::developer::DeveloperCredentialIdentity {
                    credential_id,
                    application_id,
                },
            );
            next.run(request).await
        }
        Ok(DeveloperCredentialValidation::ScopeDenied) => {
            AppError::from(DeveloperError::CredentialScopeDenied).into_response()
        }
        Ok(DeveloperCredentialValidation::RateLimited) => {
            AppError::AuthenticationRateLimited.into_response()
        }
        Ok(DeveloperCredentialValidation::Invalid) => {
            AppError::from(DeveloperError::CredentialInvalid).into_response()
        }
        Err(error) => error.into_response(),
    }
}

fn read_developer_bearer_credential(request: &Request) -> Option<&str> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix(DEVELOPER_CREDENTIAL_AUTHORIZATION_PREFIX)?;
    let secret = token.strip_prefix(DEVELOPER_CREDENTIAL_PREFIX)?;
    if secret.len() != DEVELOPER_CREDENTIAL_SECRET_LENGTH
        || !secret
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, b'_' | b'-'))
    {
        return None;
    }
    Some(token)
}

fn get_runtime_credential_environment(
    production_environment: bool,
) -> DeveloperCredentialEnvironment {
    if production_environment {
        DeveloperCredentialEnvironment::Production
    } else {
        DeveloperCredentialEnvironment::Sandbox
    }
}

pub fn developer_credential_guard(
    state: &AppState,
    permission_scope: &'static str,
) -> DeveloperCredentialGuard {
    DeveloperCredentialGuard::new(
        state.database.clone(),
        state.production_environment,
        permission_scope,
    )
}
