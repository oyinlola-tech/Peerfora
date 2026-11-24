use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    controllers::communication_controller::notification_handlers,
    dtos::communication::{
        ConversationMemberResponse, ConversationPageResponse, ConversationResponse, MessageResponse,
    },
    errors::app_error::AppError,
    models::{
        app_state::AppState,
        communication::{
            ConversationCreateInput, ConversationMemberAddInput, ConversationReportInput,
            ConversationUpdateInput, MessageCreateInput, MessageReportInput, NotificationCategory,
        },
    },
    services::{
        messaging::{
            commands::{
                conversation_commands,
                conversation_commands::{
                    add_conversation_members as add_conversation_members_cmd,
                    remove_conversation_member as remove_conversation_member_cmd,
                    report_conversation as report_conversation_cmd, send_message,
                },
                message_commands::{
                    mark_message_read as mark_message_read_cmd,
                    report_message as report_message_cmd,
                },
            },
            queries::messaging_queries,
        },
        notifications::service,
    },
};

#[derive(Deserialize)]
pub struct ConversationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct MessageQuery {
    pub before_id: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn list_conversations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ConversationQuery>,
) -> Result<Json<ConversationPageResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let (conversations, total_count) = messaging_queries::list_conversations(
        &state.database,
        account_id,
        query.limit,
        query.offset,
    )
    .await?;
    let page_size = conversations.len() as i64;
    Ok(Json(ConversationPageResponse {
        conversations: conversations
            .into_iter()
            .map(ConversationResponse::from)
            .collect(),
        total_count,
        page_size,
        offset: query.offset.unwrap_or(0),
    }))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ConversationCreateInput>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let conversation_id = conversation_commands::create_conversation(
        &state.database,
        account_id,
        &request.conversation_type,
        &request.title,
        request.research_project_id,
        &request.member_account_ids,
    )
    .await?;
    let conversation =
        messaging_queries::get_conversation(&state.database, account_id, conversation_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(ConversationResponse::from(conversation)),
    ))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<ConversationResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let conversation =
        messaging_queries::get_conversation(&state.database, account_id, conversation_id).await?;
    Ok(Json(ConversationResponse::from(conversation)))
}

pub async fn update_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<ConversationUpdateInput>,
) -> Result<Json<ConversationResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    conversation_commands::update_conversation(
        &state.database,
        account_id,
        conversation_id,
        &request.title,
    )
    .await?;
    let conversation =
        messaging_queries::get_conversation(&state.database, account_id, conversation_id).await?;
    Ok(Json(ConversationResponse::from(conversation)))
}

pub async fn list_conversation_members(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Vec<ConversationMemberResponse>>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let members =
        messaging_queries::list_conversation_members(&state.database, account_id, conversation_id)
            .await?;
    Ok(Json(
        members
            .into_iter()
            .map(ConversationMemberResponse::from)
            .collect(),
    ))
}

pub async fn add_conversation_members(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<ConversationMemberAddInput>,
) -> Result<Json<Vec<ConversationMemberResponse>>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    add_conversation_members_cmd(
        &state.database,
        account_id,
        conversation_id,
        &request.account_ids,
    )
    .await?;
    let members =
        messaging_queries::list_conversation_members(&state.database, account_id, conversation_id)
            .await?;
    Ok(Json(
        members
            .into_iter()
            .map(ConversationMemberResponse::from)
            .collect(),
    ))
}

pub async fn remove_conversation_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conversation_id, target_account_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    remove_conversation_member_cmd(
        &state.database,
        account_id,
        conversation_id,
        target_account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_conversation_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let messages = messaging_queries::list_messages(
        &state.database,
        account_id,
        conversation_id,
        query.before_id,
        query.limit,
    )
    .await?;
    Ok(Json(
        messages.into_iter().map(MessageResponse::from).collect(),
    ))
}

pub async fn send_conversation_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<MessageCreateInput>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let (message, recipient_ids) = send_message(
        &state.database,
        account_id,
        conversation_id,
        &request.body,
        request.parent_message_id,
        Some(&*state.realtime_hub),
    )
    .await?;
    if let Some(encryption_key) = state.admin_mfa_encryption_key.as_deref() {
        for recipient_id in recipient_ids {
            let recipient_email: Option<String> = sqlx::query_scalar(
                "SELECT email FROM accounts WHERE id = $1 AND account_status = 'active'",
            )
            .bind(recipient_id)
            .fetch_optional(&state.database)
            .await
            .map_err(|_| AppError::Internal)?;
            if let Some(recipient_email) = recipient_email {
                service::enqueue_notification_email(
                    &state.database,
                    recipient_id,
                    &recipient_email,
                    encryption_key,
                    NotificationCategory::Message,
                    &message.body,
                    &message.body,
                )
                .await?;
            }
        }
    }
    Ok((StatusCode::CREATED, Json(MessageResponse::from(message))))
}

pub async fn get_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let message = messaging_queries::get_message(&state.database, account_id, message_id).await?;
    Ok(Json(MessageResponse::from(message)))
}

pub async fn mark_message_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    mark_message_read_cmd(&state.database, account_id, message_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn report_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<ConversationReportInput>,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    report_conversation_cmd(
        &state.database,
        account_id,
        conversation_id,
        &request.category,
        &request.description,
    )
    .await?;
    Ok(StatusCode::CREATED)
}

pub async fn report_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<Uuid>,
    Json(request): Json<MessageReportInput>,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    report_message_cmd(
        &state.database,
        account_id,
        message_id,
        &request.category,
        &request.description,
    )
    .await?;
    Ok(StatusCode::CREATED)
}

pub(crate) async fn require_account_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Uuid, AppError> {
    notification_handlers::require_account_id(state, headers).await
}
