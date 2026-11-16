use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    constants::agents,
    controllers::research_auth::require_authenticated_account_id,
    dtos::agent_credential::{
        AgentCredentialItem, AgentCredentialListResponse, IssueAgentCredentialRequest,
        IssuedAgentCredentialResponse, RotateAgentCredentialRequest,
    },
    enums::agent::AgentCredentialEnvironment,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::agents::{
        commands::{
            issue_agent_credential_command, revoke_agent_credential_command,
            rotate_agent_credential_command,
        },
        queries::list_agent_credentials_query,
    },
};

pub async fn issue_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<IssueAgentCredentialRequest>,
) -> Result<Response, AppError> {
    let operator_account_id = require_authenticated_account_id(&state, &headers).await?;
    let issued = issue_agent_credential_command::issue_agent_credential(
        &state.database,
        operator_account_id,
        agent_id,
        &request.name,
        request.capabilities,
        request.expires_in_days,
        get_runtime_credential_environment(state.production_environment),
    )
    .await?;
    let response = IssuedAgentCredentialResponse {
        credential: issued.record.try_into()?,
        secret: issued.secret,
    };
    Ok(secret_response(StatusCode::CREATED, response))
}

pub async fn list_credentials(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentCredentialListResponse>, AppError> {
    let operator_account_id = require_authenticated_account_id(&state, &headers).await?;
    let records = list_agent_credentials_query::list_agent_credentials(
        &state.database,
        operator_account_id,
        agent_id,
    )
    .await?;
    let items = records
        .into_iter()
        .map(AgentCredentialItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(AgentCredentialListResponse { items }))
}

pub async fn rotate_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((agent_id, credential_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<RotateAgentCredentialRequest>,
) -> Result<Response, AppError> {
    let operator_account_id = require_authenticated_account_id(&state, &headers).await?;
    let rotated = rotate_agent_credential_command::rotate_agent_credential(
        &state.database,
        operator_account_id,
        agent_id,
        credential_id,
        request.expires_in_days,
        get_runtime_credential_environment(state.production_environment),
    )
    .await?;
    let response = IssuedAgentCredentialResponse {
        credential: rotated.record.try_into()?,
        secret: rotated.secret,
    };
    Ok(secret_response(StatusCode::CREATED, response))
}

pub async fn revoke_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((agent_id, credential_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let operator_account_id = require_authenticated_account_id(&state, &headers).await?;
    revoke_agent_credential_command::revoke_agent_credential(
        &state.database,
        operator_account_id,
        agent_id,
        credential_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn secret_response<T: serde::Serialize>(status: StatusCode, response: T) -> Response {
    let mut result = (status, Json(response)).into_response();
    result.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(agents::AGENT_CREDENTIAL_SECRET_CACHE_CONTROL),
    );
    result
}

fn get_runtime_credential_environment(production_environment: bool) -> AgentCredentialEnvironment {
    if production_environment {
        AgentCredentialEnvironment::Production
    } else {
        AgentCredentialEnvironment::Sandbox
    }
}
