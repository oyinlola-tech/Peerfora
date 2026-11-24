use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{
        CONVERSATION_DEFAULT_PAGE_SIZE, CONVERSATION_MAX_PAGE_OFFSET, CONVERSATION_MAX_PAGE_SIZE,
        MESSAGE_DEFAULT_PAGE_SIZE,
    },
    errors::app_error::AppError,
    models::communication::{ConversationMemberSummary, ConversationSummary, MessageSummary},
    repositories::communication_repository::{conversation_repository, message_repository},
};

pub async fn list_conversations(
    pool: &PgPool,
    account_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(Vec<ConversationSummary>, i64), AppError> {
    let limit = limit
        .unwrap_or(CONVERSATION_DEFAULT_PAGE_SIZE)
        .clamp(1, CONVERSATION_MAX_PAGE_SIZE);
    let offset = offset.unwrap_or(0).clamp(0, CONVERSATION_MAX_PAGE_OFFSET);
    conversation_repository::list_conversations(pool, account_id, limit, offset).await
}

pub async fn get_conversation(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
) -> Result<ConversationSummary, AppError> {
    conversation_repository::get_conversation(pool, account_id, conversation_id).await
}

pub async fn list_conversation_members(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
) -> Result<Vec<ConversationMemberSummary>, AppError> {
    conversation_repository::list_members(pool, account_id, conversation_id).await
}

pub async fn list_messages(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    before_id: Option<Uuid>,
    limit: Option<i64>,
) -> Result<Vec<MessageSummary>, AppError> {
    conversation_repository::require_active_membership(pool, account_id, conversation_id).await?;
    message_repository::list_messages(
        pool,
        account_id,
        conversation_id,
        before_id,
        limit.unwrap_or(MESSAGE_DEFAULT_PAGE_SIZE),
    )
    .await
}

pub async fn get_message(
    pool: &PgPool,
    account_id: Uuid,
    message_id: Uuid,
) -> Result<MessageSummary, AppError> {
    message_repository::get_message(pool, account_id, message_id).await
}
