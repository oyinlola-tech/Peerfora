use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::NOTIFICATION_RESOURCE_TYPE_CONVERSATION,
    enums::account_lifecycle::AccountLifecycleStatus,
    enums::communication::{ConversationLifecycleStatus, MembershipStatus},
    enums::phase_four::CommunicationError,
    errors::app_error::AppError,
    repositories::communication_repository::{conversation_repository, message_repository},
};

pub async fn create_conversation(
    pool: &PgPool,
    actor_account_id: Uuid,
    conversation_type: &str,
    title: &Option<String>,
    research_project_id: Option<Uuid>,
    member_account_ids: &[Uuid],
) -> Result<Uuid, AppError> {
    conversation_repository::create_conversation(
        pool,
        actor_account_id,
        conversation_type,
        title,
        research_project_id,
        member_account_ids,
    )
    .await
}

pub async fn update_conversation(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    title: &Option<String>,
) -> Result<(), AppError> {
    conversation_repository::require_active_membership(pool, account_id, conversation_id).await?;
    conversation_repository::validate_conversation_title(title)?;
    let active = ConversationLifecycleStatus::Active.as_db_value();
    let archived = ConversationLifecycleStatus::Archived.as_db_value();
    sqlx::query("UPDATE conversations SET title = $3, updated_at = now() WHERE id = $1 AND owner_account_id = $2 AND lifecycle_status IN ($4, $5) RETURNING id")
        .bind(conversation_id)
        .bind(account_id)
        .bind(title.clone())
        .bind(active)
        .bind(archived)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(CommunicationError::ConversationMembershipDenied.into())?;
    Ok(())
}

pub async fn add_conversation_members(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    account_ids: &[Uuid],
) -> Result<(), AppError> {
    conversation_repository::require_active_membership(pool, account_id, conversation_id).await?;
    if account_ids.is_empty()
        || account_ids.len() > crate::constants::phase_four::CONVERSATION_MAX_MEMBERS as usize
    {
        return Err(CommunicationError::ConversationMemberInvalid.into());
    }
    let active = MembershipStatus::Active.as_db_value();
    let inserted = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO conversation_members (conversation_id, account_id, membership_status)\
         SELECT $1, account.id, $3 FROM accounts account\
         WHERE account.id = ANY($2::UUID[]) AND account.account_status = $4\
         AND NOT EXISTS (SELECT 1 FROM user_blocks block\
             WHERE block.blocker_account_id = account.id OR block.blocked_account_id = account.id)\
         ON CONFLICT DO NOTHING RETURNING account_id",
    )
    .bind(conversation_id)
    .bind(account_ids)
    .bind(active)
    .bind(AccountLifecycleStatus::Active.as_database_value())
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if inserted.len() != account_ids.len() {
        return Err(CommunicationError::ConversationMemberInvalid.into());
    }
    conversation_repository::touch_conversation(pool, conversation_id).await
}

pub async fn remove_conversation_member(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    target_account_id: Uuid,
) -> Result<(), AppError> {
    conversation_repository::require_active_membership(pool, account_id, conversation_id).await?;
    let removed = MembershipStatus::Removed.as_db_value();
    let active = MembershipStatus::Active.as_db_value();
    let result = sqlx::query(
        "UPDATE conversation_members SET membership_status = $3, updated_at = now()\
         WHERE conversation_id = $1 AND account_id = $2 AND membership_status = $4\
         RETURNING conversation_id",
    )
    .bind(conversation_id)
    .bind(target_account_id)
    .bind(removed)
    .bind(active)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.is_some() {
        conversation_repository::touch_conversation(pool, conversation_id).await
    } else {
        Err(CommunicationError::ConversationMembershipDenied.into())
    }
}

pub async fn set_conversation_lifecycle(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    status: ConversationLifecycleStatus,
) -> Result<(), AppError> {
    conversation_repository::require_active_membership(pool, account_id, conversation_id).await?;
    conversation_repository::set_lifecycle_status(pool, conversation_id, status).await
}

pub async fn report_conversation(
    pool: &PgPool,
    reporter_account_id: Uuid,
    conversation_id: Uuid,
    category: &str,
    description: &str,
) -> Result<(), AppError> {
    let parsed_category =
        crate::enums::moderation::ModerationReportCategory::from_category(category)?;
    crate::services::moderation::commands::create_moderation_report_command::create_moderation_report(
        pool,
        reporter_account_id,
        crate::models::moderation::ModerationReportTarget::Conversation(conversation_id),
        parsed_category,
        description,
    )
    .await?;
    Ok(())
}

pub async fn send_message(
    pool: &PgPool,
    sender_account_id: Uuid,
    conversation_id: Uuid,
    body: &str,
    parent_message_id: Option<Uuid>,
    hub: Option<&crate::services::realtime::hub::RealtimeHub>,
) -> Result<(crate::models::communication::MessageSummary, Vec<Uuid>), AppError> {
    let message = message_repository::send_message(
        pool,
        sender_account_id,
        conversation_id,
        body,
        parent_message_id,
    )
    .await?;
    conversation_repository::touch_conversation(pool, conversation_id).await?;
    let recipient_ids = crate::services::notifications::service::notification_recipients(
        pool,
        conversation_id,
        sender_account_id,
    )
    .await?;
    for recipient_id in &recipient_ids {
        crate::services::notifications::service::emit_notification(
            pool,
            *recipient_id,
            crate::models::communication::NotificationCategory::Message,
            &message.body,
            &message.body,
            NOTIFICATION_RESOURCE_TYPE_CONVERSATION,
            Some(conversation_id),
        )
        .await?;
        if let Some(hub) = hub {
            crate::services::notifications::service::publish_notification_realtime(
                hub,
                *recipient_id,
                Uuid::new_v4(),
                Some(conversation_id),
            );
        }
    }
    Ok((message, recipient_ids))
}
