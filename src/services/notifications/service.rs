use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{
        NOTIFICATION_RESOURCE_TYPE_CONVERSATION, OUTBOX_AGGREGATE_TYPE_NOTIFICATION,
    },
    enums::communication::{
        MembershipStatus, NotificationCategory, NotificationEventType, RealtimeEventType,
    },
    errors::app_error::AppError,
    repositories::communication_repository::{notification_repository, outbox_repository},
    services::notifications::commands::create_notification_command,
    services::realtime::hub::RealtimeHub,
    utils::encrypted_secret::encrypt_secret,
};

/// Creates an in-app notification and a durable outbox event. Delivery of the
/// email/push intent happens in the delivery worker; in-app visibility is
/// immediate and independent of that worker.
pub async fn emit_notification(
    pool: &PgPool,
    recipient_account_id: Uuid,
    category: NotificationCategory,
    title: &str,
    body: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
) -> Result<Uuid, AppError> {
    let notification_id = create_notification_command::create_notification(
        pool,
        recipient_account_id,
        category.as_db_value(),
        title,
        body,
        resource_type,
        resource_id,
    )
    .await?;
    let payload = serde_json::json!({
        "recipient_account_id": recipient_account_id,
        "notification_id": notification_id,
        "category": category.as_db_value(),
        "title": title,
        "body": body,
        "resource_type": resource_type,
        "resource_id": resource_id,
    });
    outbox_repository::publish_outbox_event(
        pool,
        NotificationEventType::NotificationCreated.as_db_value(),
        OUTBOX_AGGREGATE_TYPE_NOTIFICATION,
        notification_id,
        &payload,
    )
    .await?;
    Ok(notification_id)
}

pub async fn emit_notification_for_members(
    pool: &PgPool,
    conversation_id: Uuid,
    sender_account_id: Uuid,
    category: NotificationCategory,
    title: &str,
    body: &str,
) -> Result<(), AppError> {
    for member_id in notification_recipients(pool, conversation_id, sender_account_id).await? {
        emit_notification(
            pool,
            member_id,
            category,
            title,
            body,
            NOTIFICATION_RESOURCE_TYPE_CONVERSATION,
            Some(conversation_id),
        )
        .await?;
    }
    Ok(())
}

pub async fn notification_recipients(
    pool: &PgPool,
    conversation_id: Uuid,
    sender_account_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    sqlx::query_scalar::<_, Uuid>(
        "SELECT account_id FROM conversation_members \
         WHERE conversation_id = $1 AND membership_status = $2 AND account_id <> $3",
    )
    .bind(conversation_id)
    .bind(active_membership)
    .bind(sender_account_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

/// Enqueues an encrypted email delivery intent for a notification. The
/// payload never contains private message content in logs or ordinary
/// outbox payloads; the email body itself is the intended recipient content.
pub async fn enqueue_notification_email(
    pool: &PgPool,
    recipient_account_id: Uuid,
    recipient_email: &str,
    encryption_key: &str,
    category: NotificationCategory,
    title: &str,
    body: &str,
) -> Result<Uuid, AppError> {
    let preferences = notification_repository::get_preferences(pool, recipient_account_id).await?;
    if !preferences.email_enabled {
        return Ok(Uuid::new_v4());
    }
    let message = serde_json::json!({
        "subject": title,
        "plain_text": body,
        "html": format!("<p>{}</p>", escape_html(body)),
        "category": category.as_db_value(),
    });
    let payload = serde_json::to_vec(&message).map_err(|_| AppError::Internal)?;
    let encrypted_payload = encrypt_secret(&payload, encryption_key)?;
    outbox_repository::publish_email_delivery(pool, recipient_email, &encrypted_payload).await
}

pub fn publish_notification_realtime(
    hub: &RealtimeHub,
    recipient_account_id: Uuid,
    notification_id: Uuid,
    conversation_id: Option<Uuid>,
) {
    let payload = serde_json::json!({
        "notification_id": notification_id,
        "conversation_id": conversation_id,
    });
    let _ = hub.publish(
        recipient_account_id,
        RealtimeEventType::NotificationCreated.as_db_value(),
        conversation_id,
        &payload,
    );
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
