use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{
        NOTIFICATION_MAX_BODY_LENGTH, NOTIFICATION_MAX_CATEGORY_LENGTH,
        NOTIFICATION_MAX_PAGE_OFFSET, NOTIFICATION_MAX_PAGE_SIZE,
        NOTIFICATION_MAX_RESOURCE_TYPE_LENGTH, NOTIFICATION_MAX_TITLE_LENGTH,
    },
    enums::phase_four::CommunicationError,
    errors::app_error::AppError,
    models::communication::NotificationPage,
};

#[derive(sqlx::FromRow)]
struct NotificationPreferenceRecord {
    account_id: Uuid,
    in_app_enabled: bool,
    email_enabled: bool,
    push_enabled: bool,
    research_enabled: bool,
    agent_enabled: bool,
    community_enabled: bool,
}

impl From<NotificationPreferenceRecord> for crate::models::communication::NotificationPreferences {
    fn from(record: NotificationPreferenceRecord) -> Self {
        Self {
            account_id: record.account_id,
            in_app_enabled: record.in_app_enabled,
            email_enabled: record.email_enabled,
            push_enabled: record.push_enabled,
            research_enabled: record.research_enabled,
            agent_enabled: record.agent_enabled,
            community_enabled: record.community_enabled,
        }
    }
}

pub fn validate_notification(
    category: &str,
    title: &str,
    body: &str,
    resource_type: &str,
) -> Result<(), CommunicationError> {
    if category.is_empty()
        || category.len() > NOTIFICATION_MAX_CATEGORY_LENGTH
        || title.is_empty()
        || title.len() > NOTIFICATION_MAX_TITLE_LENGTH
        || body.len() > NOTIFICATION_MAX_BODY_LENGTH
        || resource_type.is_empty()
        || resource_type.len() > NOTIFICATION_MAX_RESOURCE_TYPE_LENGTH
    {
        return Err(CommunicationError::NotificationInvalid);
    }
    Ok(())
}

pub async fn create_notification(
    pool: &PgPool,
    recipient_account_id: Uuid,
    category: &str,
    title: &str,
    body: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
) -> Result<Uuid, AppError> {
    validate_notification(category, title, body, resource_type)?;
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO notifications (recipient_account_id, category, title, body, resource_type, resource_id)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(recipient_account_id)
    .bind(category)
    .bind(title)
    .bind(body)
    .bind(resource_type)
    .bind(resource_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_notifications(
    pool: &PgPool,
    account_id: Uuid,
    include_read: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<NotificationPage>, i64), AppError> {
    if !(1..=NOTIFICATION_MAX_PAGE_SIZE).contains(&limit)
        || offset.is_negative()
        || offset > NOTIFICATION_MAX_PAGE_OFFSET
    {
        return Err(CommunicationError::NotificationInvalid.into());
    }
    let predicate = if include_read {
        "1 = 1"
    } else {
        "notification.is_read = FALSE"
    };
    let query = format!(
        "SELECT notification.id, notification.recipient_account_id, notification.category,\
            notification.title, notification.body, notification.resource_type,\
            notification.resource_id, notification.is_read, notification.read_at,\
            notification.created_at, count(*) OVER() AS total_count\
         FROM notifications notification\
         WHERE notification.recipient_account_id = $1 AND {predicate}\
         ORDER BY notification.created_at DESC, notification.id DESC LIMIT $2 OFFSET $3"
    );
    let rows = sqlx::query_as::<_, NotificationPageWithCount>(&query)
        .bind(account_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    let total = rows.first().map(|row| row.total_count).unwrap_or(0);
    Ok((rows.into_iter().map(Into::into).collect(), total))
}

#[derive(sqlx::FromRow)]
struct NotificationPageWithCount {
    id: Uuid,
    recipient_account_id: Uuid,
    category: String,
    title: String,
    body: String,
    resource_type: String,
    resource_id: Option<Uuid>,
    is_read: bool,
    read_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    total_count: i64,
}

impl From<NotificationPageWithCount> for NotificationPage {
    fn from(record: NotificationPageWithCount) -> Self {
        Self {
            id: record.id,
            recipient_account_id: record.recipient_account_id,
            category: record.category,
            title: record.title,
            body: record.body,
            resource_type: record.resource_type,
            resource_id: record.resource_id,
            is_read: record.is_read,
            read_at: record.read_at,
            created_at: record.created_at,
        }
    }
}

pub async fn unread_notification_count(pool: &PgPool, account_id: Uuid) -> Result<i64, AppError> {
    sqlx::query_scalar("SELECT count(*)::BIGINT FROM notifications WHERE recipient_account_id = $1 AND is_read = FALSE")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn read_notification(
    pool: &PgPool,
    account_id: Uuid,
    notification_id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query_scalar::<_, bool>(
        "UPDATE notifications SET is_read = TRUE, read_at = now()
         WHERE id = $1 AND recipient_account_id = $2 AND is_read = FALSE RETURNING TRUE",
    )
    .bind(notification_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.is_some() {
        return Ok(true);
    }
    if notification_exists(pool, account_id, notification_id).await? {
        return Ok(false);
    }
    Err(CommunicationError::NotificationNotFound.into())
}

async fn notification_exists(
    pool: &PgPool,
    account_id: Uuid,
    notification_id: Uuid,
) -> Result<bool, AppError> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM notifications WHERE id = $1 AND recipient_account_id = $2)",
    )
    .bind(notification_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn read_all_notifications(pool: &PgPool, account_id: Uuid) -> Result<i64, AppError> {
    sqlx::query_scalar("UPDATE notifications SET is_read = TRUE, read_at = now() WHERE recipient_account_id = $1 AND is_read = FALSE RETURNING id")
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.len() as i64)
        .map_err(|_| AppError::Internal)
}

pub async fn get_preferences(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<crate::models::communication::NotificationPreferences, AppError> {
    let record = sqlx::query_as::<_, NotificationPreferenceRecord>(
        "SELECT account_id, in_app_enabled, email_enabled, push_enabled,\
            research_enabled, agent_enabled, community_enabled\
         FROM notification_preferences WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::NotificationPreferencesInvalid)?;
    Ok(record.into())
}

pub async fn update_preferences(
    pool: &PgPool,
    account_id: Uuid,
    in_app_enabled: bool,
    email_enabled: bool,
    push_enabled: bool,
    research_enabled: bool,
    agent_enabled: bool,
    community_enabled: bool,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO notification_preferences (account_id, in_app_enabled, email_enabled,\
            push_enabled, research_enabled, agent_enabled, community_enabled, updated_at)\
         VALUES ($1, $2, $3, $4, $5, $6, $7, now())\
         ON CONFLICT (account_id) DO UPDATE SET in_app_enabled = EXCLUDED.in_app_enabled,\
            email_enabled = EXCLUDED.email_enabled, push_enabled = EXCLUDED.push_enabled,\
            research_enabled = EXCLUDED.research_enabled, agent_enabled = EXCLUDED.agent_enabled,\
            community_enabled = EXCLUDED.community_enabled, updated_at = now()",
    )
    .bind(account_id)
    .bind(in_app_enabled)
    .bind(email_enabled)
    .bind(push_enabled)
    .bind(research_enabled)
    .bind(agent_enabled)
    .bind(community_enabled)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
