use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::NOTIFICATION_RESOURCE_TYPE_CONVERSATION,
    enums::communication::ConversationLifecycleStatus,
    errors::app_error::AppError,
    repositories::communication_repository::{
        conversation_membership_repository, conversation_repository, message_repository,
    },
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
    conversation_repository::validate_conversation_title(title)?;
    conversation_membership_repository::update_conversation_title(
        pool,
        account_id,
        conversation_id,
        title,
    )
    .await
}

pub async fn add_conversation_members(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    account_ids: &[Uuid],
) -> Result<(), AppError> {
    if account_ids.is_empty()
        || account_ids.len() > crate::constants::phase_four::CONVERSATION_MAX_MEMBERS as usize
    {
        return Err(crate::enums::phase_four::CommunicationError::ConversationMemberInvalid.into());
    }
    conversation_membership_repository::add_conversation_members(
        pool,
        account_id,
        conversation_id,
        account_ids,
    )
    .await
}

pub async fn remove_conversation_member(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    target_account_id: Uuid,
) -> Result<(), AppError> {
    conversation_membership_repository::remove_conversation_member(
        pool,
        account_id,
        conversation_id,
        target_account_id,
    )
    .await
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
            "New message",
            "You received a new message.",
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
