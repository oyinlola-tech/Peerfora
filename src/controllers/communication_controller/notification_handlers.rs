use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    constants::account_auth::SESSION_COOKIE_NAME,
    controllers::auth_cookie,
    dtos::communication::{
        DevicePushTokenResponse, NotificationPageResponse, NotificationPreferencesResponse,
        NotificationResponse, UnreadNotificationCountResponse,
    },
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::{
        app_state::AppState,
        communication::{
            DevicePushTokenInput, DevicePushTokenUpdate, NotificationPreferencesInput,
        },
    },
    services::{
        accounts::queries::account_session_queries,
        notifications::{
            commands::{read_notification_command, update_notification_preferences_command},
            queries::list_notifications_query,
        },
        push::{commands::device_push_commands, queries::device_push_queries},
    },
};

#[derive(Deserialize)]
pub struct NotificationQuery {
    pub include_read: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<NotificationPageResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let (notifications, total_count) = list_notifications_query::list_notifications(
        &state.database,
        account_id,
        query.include_read.unwrap_or(false),
        query.limit,
        query.offset,
    )
    .await?;
    Ok(Json(NotificationPageResponse {
        notifications: notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect(),
        total_count,
        page_size: notifications.len() as i64,
        offset: query.offset.unwrap_or(0),
    }))
}

pub async fn unread_notification_count(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UnreadNotificationCountResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let unread_count =
        list_notifications_query::unread_notification_count(&state.database, account_id).await?;
    Ok(Json(UnreadNotificationCountResponse { unread_count }))
}

pub async fn read_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(notification_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    read_notification_command::read_notification(&state.database, account_id, notification_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn read_all_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UnreadNotificationCountResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    read_notification_command::read_all_notifications(&state.database, account_id).await?;
    Ok(Json(UnreadNotificationCountResponse { unread_count: 0 }))
}

pub async fn get_notification_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<NotificationPreferencesResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let preferences =
        list_notifications_query::get_notification_preferences(&state.database, account_id).await?;
    Ok(Json(NotificationPreferencesResponse(preferences)))
}

pub async fn update_notification_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<NotificationPreferencesInput>,
) -> Result<Json<NotificationPreferencesResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    update_notification_preferences_command::update_notification_preferences(
        &state.database,
        account_id,
        request.in_app_enabled,
        request.email_enabled,
        request.push_enabled,
        request.research_enabled,
        request.agent_enabled,
        request.community_enabled,
    )
    .await?;
    let preferences =
        list_notifications_query::get_notification_preferences(&state.database, account_id).await?;
    Ok(Json(NotificationPreferencesResponse(preferences)))
}

pub async fn register_device_push_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DevicePushTokenInput>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let token_id = device_push_commands::register_device_token(
        &state.database,
        account_id,
        &request.platform,
        &request.token,
    )
    .await?;
    let token =
        device_push_queries::get_device_token(&state.database, account_id, token_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(DevicePushTokenResponse::from(token)),
    ))
}

pub async fn update_device_push_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token_id): Path<Uuid>,
    Json(request): Json<DevicePushTokenUpdate>,
) -> Result<Json<DevicePushTokenResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    device_push_commands::update_device_token(
        &state.database,
        account_id,
        token_id,
        &request.platform,
        request.active,
    )
    .await?;
    let token =
        device_push_queries::get_device_token(&state.database, account_id, token_id).await?;
    Ok(Json(DevicePushTokenResponse::from(token)))
}

pub async fn list_device_push_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DevicePushTokenResponse>>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let tokens = device_push_queries::list_device_tokens(&state.database, account_id).await?;
    Ok(Json(
        tokens
            .into_iter()
            .map(DevicePushTokenResponse::from)
            .collect(),
    ))
}

pub async fn get_device_push_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token_id): Path<Uuid>,
) -> Result<Json<DevicePushTokenResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let token =
        device_push_queries::get_device_token(&state.database, account_id, token_id).await?;
    Ok(Json(DevicePushTokenResponse::from(token)))
}

pub(crate) async fn require_account_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Uuid, AppError> {
    let token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(token) = token else {
        return Err(AccountAuthError::SessionInvalid.into());
    };
    let account = account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .ok_or(AccountAuthError::SessionInvalid)?;
    Ok(account.id)
}
