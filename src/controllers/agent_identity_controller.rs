use crate::{
    constants::account_auth::SESSION_COOKIE_NAME,
    controllers::auth_cookie,
    dtos::agent_identity::{AgentIdentityResponse, ListMyAgentsQuery, RegisterAgentRequest},
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::{
        accounts::queries::account_session_queries,
        agents::{
            commands::register_agent_command::{self, AgentRegistrationDetails},
            queries::list_my_agents_query,
        },
    },
};
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};

pub async fn register_agent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RegisterAgentRequest>,
) -> Result<(StatusCode, Json<AgentIdentityResponse>), AppError> {
    let operator_account = find_operator_account(&state, &headers).await?;
    crate::services::access_control::commands::enforce_authentication_rate_limit_command::enforce_authentication_rate_limit(
        &state.database,
        crate::enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation::AgentRegistration,
        &operator_account.email,
        &state.authentication_rate_limit_hash_key,
    )
    .await?;
    let agent = register_agent_command::register_agent(
        &state.database,
        operator_account.id,
        AgentRegistrationDetails {
            handle: request.handle,
            display_name: request.display_name,
            description: request.description,
            model_disclosure: request.model_disclosure,
            specialization_subject_ids: request.specialization_subject_ids,
            requested_capabilities: request.requested_capabilities,
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(AgentIdentityResponse::try_from(agent)?),
    ))
}

pub async fn list_my_agents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListMyAgentsQuery>,
) -> Result<Json<Vec<AgentIdentityResponse>>, AppError> {
    let operator_account = find_operator_account(&state, &headers).await?;
    let agents =
        list_my_agents_query::list_my_agents(&state.database, operator_account.id, query.limit)
            .await?;
    Ok(Json(agents))
}

async fn find_operator_account(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::models::account::Account, AppError> {
    let token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(token) = token else {
        return Err(AccountAuthError::SessionInvalid.into());
    };

    account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .ok_or(AccountAuthError::SessionInvalid.into())
}
