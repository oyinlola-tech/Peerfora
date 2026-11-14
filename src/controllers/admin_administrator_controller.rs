use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dtos::admin_administrator::AdministratorLifecycleRequest,
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::administration::commands::{
        delete_administrator_command::delete_administrator,
        reinstate_administrator_command::reinstate_administrator,
        reset_administrator_mfa_command::reset_administrator_mfa,
        revoke_administrator_sessions_command::revoke_administrator_sessions,
        suspend_administrator_command::suspend_administrator,
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn suspend_administrator_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(target_id): Path<Uuid>,
    Json(request): Json<AdministratorLifecycleRequest>,
) -> Result<impl IntoResponse, AppError> {
    suspend_administrator(
        &state.database,
        actor.administrator_id,
        actor.session_id,
        target_id,
        request.reason.as_str(),
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reinstate_administrator_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(target_id): Path<Uuid>,
    Json(request): Json<AdministratorLifecycleRequest>,
) -> Result<impl IntoResponse, AppError> {
    reinstate_administrator(
        &state.database,
        actor.administrator_id,
        actor.session_id,
        target_id,
        request.reason.as_str(),
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_administrator_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(target_id): Path<Uuid>,
    Json(request): Json<AdministratorLifecycleRequest>,
) -> Result<impl IntoResponse, AppError> {
    delete_administrator(
        &state.database,
        actor.administrator_id,
        actor.session_id,
        target_id,
        request.reason.as_str(),
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_administrator_account_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(target_id): Path<Uuid>,
    Json(request): Json<AdministratorLifecycleRequest>,
) -> Result<impl IntoResponse, AppError> {
    revoke_administrator_sessions(
        &state.database,
        actor.administrator_id,
        actor.session_id,
        target_id,
        request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reset_administrator_mfa_factor(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(actor): Extension<AdministratorSessionIdentity>,
    Path(target_id): Path<Uuid>,
    Json(request): Json<AdministratorLifecycleRequest>,
) -> Result<impl IntoResponse, AppError> {
    reset_administrator_mfa(
        &state.database,
        actor.administrator_id,
        actor.session_id,
        target_id,
        request.reason.as_str(),
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
