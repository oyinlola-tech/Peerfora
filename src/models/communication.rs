use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::enums::communication::{ConversationLifecycleStatus, NotificationCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationTargetType {
    Conversation,
    Message,
    Media,
}

impl CommunicationTargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::Message => "message",
            Self::Media => "media",
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct NotificationPage {
    pub id: Uuid,
    pub recipient_account_id: Uuid,
    pub category: String,
    pub title: String,
    pub body: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub conversation_type: String,
    pub title: Option<String>,
    pub research_project_id: Option<Uuid>,
    pub owner_account_id: Uuid,
    pub lifecycle_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_message_id: Option<Uuid>,
    pub last_message_created_at: Option<DateTime<Utc>>,
    pub member_count: i64,
    pub unread_count: i64,
    pub is_restricted: bool,
    pub is_closed: bool,
}

#[derive(sqlx::FromRow)]
pub struct ConversationMemberSummary {
    pub conversation_id: Uuid,
    pub account_id: Uuid,
    pub member_role: String,
    pub membership_status: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct MessageSummary {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_account_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub mine: bool,
    pub read_by_count: i64,
    pub received_count: i64,
}

#[derive(sqlx::FromRow)]
pub struct MediaObjectSummary {
    pub id: Uuid,
    pub owner_account_id: Uuid,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub object_key: String,
    pub original_filename: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub checksum: String,
    pub processing_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authorization_resource: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct DevicePushTokenSummary {
    pub id: Uuid,
    pub account_id: Uuid,
    pub platform: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct MediaProcessingJobSummary {
    pub id: Uuid,
    pub media_object_id: Uuid,
    pub status: String,
    pub attempt_count: i32,
    pub available_at: DateTime<Utc>,
    pub last_error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationPreferencesInput {
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub research_enabled: bool,
    pub agent_enabled: bool,
    pub community_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationPreferences {
    pub account_id: Uuid,
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub research_enabled: bool,
    pub agent_enabled: bool,
    pub community_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct DevicePushTokenInput {
    pub platform: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct DevicePushTokenUpdate {
    pub platform: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConversationCreateInput {
    pub conversation_type: String,
    pub title: Option<String>,
    pub research_project_id: Option<Uuid>,
    pub member_account_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationUpdateInput {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationMemberAddInput {
    pub account_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MessageCreateInput {
    pub body: String,
    pub parent_message_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MessageReadInput {
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct MessageReportInput {
    pub category: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaUploadInitInput {
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub original_filename: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub checksum: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaUploadCompleteInput {
    pub checksum: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaReportInput {
    pub category: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ConversationReportInput {
    pub category: String,
    pub description: String,
}
