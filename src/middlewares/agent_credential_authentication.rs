use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::{
    constants::agents::{
        AGENT_CREDENTIAL_AUTHORIZATION_PREFIX, AGENT_CREDENTIAL_PREFIX,
        AGENT_CREDENTIAL_SECRET_LENGTH,
    },
    enums::agent::{AgentCapability, AgentCredentialEnvironment, AgentCredentialValidation},
    enums::agent_error::AgentError,
    errors::app_error::AppError,
    models::app_state::AppState,
    repositories::agent_credential_repository,
    utils::one_time_token::hash_one_time_token,
};

#[derive(Clone)]
pub struct AgentCredentialGuard {
    pub database: PgPool,
    pub production_environment: bool,
    pub required_capability: AgentCapability,
}

pub fn agent_credential_guard(
    state: &AppState,
    required_capability: AgentCapability,
) -> AgentCredentialGuard {
    AgentCredentialGuard {
        database: state.database.clone(),
        production_environment: state.production_environment,
        required_capability,
    }
}

pub async fn require_agent_credential(
    State(guard): State<AgentCredentialGuard>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) = read_agent_bearer_credential(&request) else {
        return AppError::from(AgentError::CredentialInvalid).into_response();
    };
    let validation = agent_credential_repository::validate_agent_credential_for_capability(
        &guard.database,
        &hash_one_time_token(token),
        get_runtime_credential_environment(guard.production_environment),
        guard.required_capability,
    )
    .await;
    match validation {
        Ok(AgentCredentialValidation::Authorized(credential_id, agent_actor_id)) => {
            request.extensions_mut().insert(
                crate::models::agent_credential::AgentCredentialIdentity {
                    credential_id,
                    agent_actor_id,
                },
            );
            next.run(request).await
        }
        Ok(AgentCredentialValidation::ScopeDenied) => {
            AppError::from(AgentError::CredentialScopeDenied).into_response()
        }
        Ok(AgentCredentialValidation::RateLimited) => {
            AppError::AuthenticationRateLimited.into_response()
        }
        Ok(AgentCredentialValidation::Invalid) => {
            AppError::from(AgentError::CredentialInvalid).into_response()
        }
        Err(error) => error.into_response(),
    }
}

fn read_agent_bearer_credential(request: &Request) -> Option<&str> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix(AGENT_CREDENTIAL_AUTHORIZATION_PREFIX)?;
    let secret = token.strip_prefix(AGENT_CREDENTIAL_PREFIX)?;
    if secret.len() != AGENT_CREDENTIAL_SECRET_LENGTH
        || !secret
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, b'_' | b'-'))
    {
        return None;
    }
    Some(token)
}

fn get_runtime_credential_environment(production_environment: bool) -> AgentCredentialEnvironment {
    if production_environment {
        AgentCredentialEnvironment::Production
    } else {
        AgentCredentialEnvironment::Sandbox
    }
}
