use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{MESSAGE_MAX_BODY_LENGTH, MESSAGE_MAX_PAGE_SIZE},
    enums::communication::{ConversationLifecycleStatus, MembershipStatus},
    enums::phase_four::CommunicationError,
    errors::app_error::AppError,
    models::communication::MessageSummary,
};

pub fn validate_message_body(body: &str) -> Result<(), CommunicationError> {
    if body.trim().is_empty() || body.chars().count() > MESSAGE_MAX_BODY_LENGTH {
        return Err(CommunicationError::MessageInvalid);
    }
    Ok(())
}

pub async fn send_message(
    pool: &PgPool,
    sender_account_id: Uuid,
    conversation_id: Uuid,
    body: &str,
    parent_message_id: Option<Uuid>,
) -> Result<MessageSummary, AppError> {
    validate_message_body(body)?;
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    sqlx::query_as::<_, MessageSummary>(
        "WITH locked_conversation AS (\
            SELECT conversation.id FROM conversations conversation\
            JOIN conversation_members sender_member ON sender_member.conversation_id = conversation.id\
                AND sender_member.account_id = $1 AND sender_member.membership_status = $2\
            WHERE conversation.id = $3 AND conversation.lifecycle_status = $4\
                AND NOT EXISTS (\
                    SELECT 1 FROM user_blocks sender_block\
                    JOIN conversation_members sender_other ON sender_other.conversation_id = conversation.id\
                        AND sender_other.account_id = sender_block.blocked_account_id\
                        AND sender_other.membership_status = $2\
                    WHERE sender_block.blocker_account_id = $1\
                )\
            FOR UPDATE OF conversation\
         )\
         INSERT INTO messages (conversation_id, sender_account_id, parent_message_id, body)\
         SELECT locked_conversation.id, $1, $5, $6\
         FROM locked_conversation\
         WHERE ($5::UUID IS NULL OR EXISTS (\
             SELECT 1 FROM messages parent WHERE parent.id = $5\
                 AND parent.conversation_id = locked_conversation.id\
         ))\
         RETURNING id, conversation_id, sender_account_id, parent_message_id, body,\
            created_at, updated_at, FALSE AS mine, 0::BIGINT AS read_by_count,\
            0::BIGINT AS received_count",
    )
    .bind(sender_account_id)
    .bind(active_membership)
    .bind(conversation_id)
    .bind(active_lifecycle)
    .bind(parent_message_id)
    .bind(body)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::ConversationBlocked.into())
}

pub async fn list_messages(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    before_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<MessageSummary>, AppError> {
    if !(1..=MESSAGE_MAX_PAGE_SIZE).contains(&limit) {
        return Err(CommunicationError::MessageInvalid.into());
    }
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let archived_lifecycle = ConversationLifecycleStatus::Archived.as_db_value();
    sqlx::query_as::<_, MessageSummary>(
        "SELECT message.id, message.conversation_id, message.sender_account_id,\
            message.parent_message_id, message.body, message.created_at, message.updated_at,\
            message.sender_account_id = $1 AS mine,\
            read_counts.read_count AS read_by_count,\
            received_counts.received_count AS received_count\
         FROM messages message\
         JOIN conversation_members membership ON membership.conversation_id = message.conversation_id\
             AND membership.account_id = $1 AND membership.membership_status = $2\
         JOIN conversations conversation ON conversation.id = message.conversation_id\
             AND conversation.lifecycle_status IN ($3, $4)\
         AND NOT EXISTS (\
             SELECT 1 FROM user_blocks block\
             JOIN conversation_members block_member ON block_member.conversation_id = message.conversation_id\
                 AND block_member.account_id = block.blocked_account_id\
                 AND block_member.membership_status = $2\
             WHERE block.blocker_account_id = $1\
         )\
         JOIN LATERAL (\
             SELECT count(*)::BIGINT AS read_count FROM message_read_states read_state\
             WHERE read_state.message_id = message.id\
         ) read_counts ON TRUE\
         JOIN LATERAL (\
             SELECT count(*)::BIGINT AS received_count FROM message_read_states received_state\
             WHERE received_state.message_id = message.id AND received_state.account_id = $1\
         ) received_counts ON TRUE\
         WHERE message.conversation_id = $5\
             AND ($6::UUID IS NULL OR (message.created_at, message.id) <\
                (SELECT created_at, id FROM messages boundary WHERE boundary.id = $6))\
         ORDER BY message.created_at DESC, message.id DESC LIMIT $7",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(active_lifecycle)
    .bind(archived_lifecycle)
    .bind(conversation_id)
    .bind(before_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .into_iter()
    .rev()
    .collect()
}

pub async fn get_message(
    pool: &PgPool,
    account_id: Uuid,
    message_id: Uuid,
) -> Result<MessageSummary, AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let archived_lifecycle = ConversationLifecycleStatus::Archived.as_db_value();
    sqlx::query_as::<_, MessageSummary>(
        "SELECT message.id, message.conversation_id, message.sender_account_id,\
            message.parent_message_id, message.body, message.created_at, message.updated_at,\
            message.sender_account_id = $1 AS mine,\
            read_counts.read_count AS read_by_count,\
            received_counts.received_count AS received_count\
         FROM messages message\
         JOIN conversation_members membership ON membership.conversation_id = message.conversation_id\
             AND membership.account_id = $1 AND membership.membership_status = $2\
         JOIN conversations conversation ON conversation.id = message.conversation_id\
             AND conversation.lifecycle_status IN ($3, $4)\
         AND NOT EXISTS (\
             SELECT 1 FROM user_blocks block\
             JOIN conversation_members block_member ON block_member.conversation_id = message.conversation_id\
                 AND block_member.account_id = block.blocked_account_id\
                 AND block_member.membership_status = $2\
             WHERE block.blocker_account_id = $1\
         )\
         JOIN LATERAL (\
             SELECT count(*)::BIGINT AS read_count FROM message_read_states read_state\
             WHERE read_state.message_id = message.id\
         ) read_counts ON TRUE\
         JOIN LATERAL (\
             SELECT count(*)::BIGINT AS received_count FROM message_read_states received_state\
             WHERE received_state.message_id = message.id AND received_state.account_id = $1\
         ) received_counts ON TRUE\
         WHERE message.id = $5",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(active_lifecycle)
    .bind(archived_lifecycle)
    .bind(message_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::MessageNotFound.into())
}

pub async fn mark_message_read(
    pool: &PgPool,
    account_id: Uuid,
    message_id: Uuid,
) -> Result<(), AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let archived_lifecycle = ConversationLifecycleStatus::Archived.as_db_value();
    let updated = sqlx::query(
        "WITH authorized AS (\
            SELECT message.id FROM messages message\
            JOIN conversation_members membership ON membership.conversation_id = message.conversation_id\
                AND membership.account_id = $1 AND membership.membership_status = $2\
            JOIN conversations conversation ON conversation.id = message.conversation_id\
                AND conversation.lifecycle_status IN ($3, $4)\
            WHERE message.id = $5\
        )\
        INSERT INTO message_read_states (message_id, account_id) SELECT id, $1 FROM authorized\
        ON CONFLICT (message_id, account_id) DO UPDATE SET read_at = now()\
        RETURNING message_id",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(active_lifecycle)
    .bind(archived_lifecycle)
    .bind(message_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if updated.is_some() {
        return Ok(());
    }
    Err(CommunicationError::MessageNotFound.into())
}

pub async fn conversation_message_count(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
) -> Result<i64, AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*)::BIGINT FROM messages message\
         JOIN conversation_members membership ON membership.conversation_id = message.conversation_id\
             AND membership.account_id = $1 AND membership.membership_status = $2\
         WHERE message.conversation_id = $3",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(conversation_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}
