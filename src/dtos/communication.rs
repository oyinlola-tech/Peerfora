use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::communication::{
    ConversationMemberSummary, ConversationSummary, DevicePushTokenSummary, MediaObjectSummary,
    MessageSummary, NotificationPage, NotificationPreferences,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationResponse {
    pub id: Uuid,
    pub category: String,
    pub title: String,
    pub body: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<NotificationPage> for NotificationResponse {
    fn from(notification: NotificationPage) -> Self {
        Self {
            id: notification.id,
            category: notification.category,
            title: notification.title,
            body: notification.body,
            resource_type: notification.resource_type,
            resource_id: notification.resource_id,
            is_read: notification.is_read,
            read_at: notification.read_at,
            created_at: notification.created_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPageResponse {
    pub notifications: Vec<NotificationResponse>,
    pub total_count: i64,
    pub page_size: i64,
    pub offset: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferencesResponse(pub NotificationPreferences);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationResponse {
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
}

impl From<ConversationSummary> for ConversationResponse {
    fn from(conversation: ConversationSummary) -> Self {
        Self {
            id: conversation.id,
            conversation_type: conversation.conversation_type,
            title: conversation.title,
            research_project_id: conversation.research_project_id,
            owner_account_id: conversation.owner_account_id,
            lifecycle_status: conversation.lifecycle_status,
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
            last_message_id: conversation.last_message_id,
            last_message_created_at: conversation.last_message_created_at,
            member_count: conversation.member_count,
            unread_count: conversation.unread_count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationPageResponse {
    pub conversations: Vec<ConversationResponse>,
    pub total_count: i64,
    pub page_size: i64,
    pub offset: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMemberResponse {
    pub conversation_id: Uuid,
    pub account_id: Uuid,
    pub member_role: String,
    pub membership_status: String,
    pub joined_at: DateTime<Utc>,
}

impl From<ConversationMemberSummary> for ConversationMemberResponse {
    fn from(member: ConversationMemberSummary) -> Self {
        Self {
            conversation_id: member.conversation_id,
            account_id: member.account_id,
            member_role: member.member_role,
            membership_status: member.membership_status,
            joined_at: member.joined_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
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

impl From<MessageSummary> for MessageResponse {
    fn from(message: MessageSummary) -> Self {
        Self {
            id: message.id,
            conversation_id: message.conversation_id,
            sender_account_id: message.sender_account_id,
            parent_message_id: message.parent_message_id,
            body: message.body,
            created_at: message.created_at,
            updated_at: message.updated_at,
            mine: message.mine,
            read_by_count: message.read_by_count,
            received_count: message.received_count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaObjectResponse {
    pub id: Uuid,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub original_filename: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub processing_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MediaObjectSummary> for MediaObjectResponse {
    fn from(media: MediaObjectSummary) -> Self {
        Self {
            id: media.id,
            resource_type: media.resource_type,
            resource_id: media.resource_id,
            original_filename: media.original_filename,
            media_type: media.media_type,
            size_bytes: media.size_bytes,
            processing_status: media.processing_status,
            created_at: media.created_at,
            updated_at: media.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaUploadInitiationResponse {
    pub media_object_id: Uuid,
    pub upload_session_id: Uuid,
    pub upload_token: String,
    pub upload_url: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePushTokenResponse {
    pub id: Uuid,
    pub platform: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DevicePushTokenSummary> for DevicePushTokenResponse {
    fn from(token: DevicePushTokenSummary) -> Self {
        Self {
            id: token.id,
            platform: token.platform,
            active: token.active,
            created_at: token.created_at,
            updated_at: token.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadNotificationCountResponse {
    pub unread_count: i64,
}
