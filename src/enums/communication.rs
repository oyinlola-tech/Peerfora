//! Typed enums for the Phase Four communication domain. Every persisted
//! status, type, role, platform, and action value is represented here so that
//! call sites never hardcode the database string literals.

/// Conversation kinds, matching the `conversations.conversation_type` CHECK.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationType {
    Direct,
    Group,
    ResearchProject,
}

impl ConversationType {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Group => "group",
            Self::ResearchProject => "research_project",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "direct" => Some(Self::Direct),
            "group" => Some(Self::Group),
            "research_project" => Some(Self::ResearchProject),
            _ => None,
        }
    }

    pub fn is_valid(value: &str) -> bool {
        Self::from_db_value(value).is_some()
    }
}

/// Conversation lifecycle states, matching `conversations.lifecycle_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationLifecycleStatus {
    Active,
    Archived,
    Restricted,
    Closed,
}

impl ConversationLifecycleStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::Restricted => "restricted",
            Self::Closed => "closed",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "archived" => Some(Self::Archived),
            "restricted" => Some(Self::Restricted),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }

    pub fn permits_messages(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// Conversation member roles, matching `conversation_members.member_role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationMemberRole {
    Member,
    Owner,
}

impl ConversationMemberRole {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::Owner => "owner",
        }
    }
}

/// Conversation membership states, matching
/// `conversation_members.membership_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipStatus {
    Active,
    Left,
    Removed,
}

impl MembershipStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Left => "left",
            Self::Removed => "removed",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "left" => Some(Self::Left),
            "removed" => Some(Self::Removed),
            _ => None,
        }
    }
}

/// Media object processing states, matching
/// `media_objects.processing_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaProcessingStatus {
    Requested,
    Uploading,
    Uploaded,
    Quarantined,
    Processing,
    Ready,
    Rejected,
    Failed,
    Deleted,
}

impl MediaProcessingStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Uploading => "uploading",
            Self::Uploaded => "uploaded",
            Self::Quarantined => "quarantined",
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
            Self::Deleted => "deleted",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "requested" => Some(Self::Requested),
            "uploading" => Some(Self::Uploading),
            "uploaded" => Some(Self::Uploaded),
            "quarantined" => Some(Self::Quarantined),
            "processing" => Some(Self::Processing),
            "ready" => Some(Self::Ready),
            "rejected" => Some(Self::Rejected),
            "failed" => Some(Self::Failed),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }

    /// States from which a media object may be quarantined by an administrator.
    pub fn quarantinable(self) -> bool {
        matches!(
            self,
            Self::Requested | Self::Uploading | Self::Uploaded | Self::Processing | Self::Ready
        )
    }

    /// Whether the stored bytes are safe to download.
    pub fn permits_download(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Media upload session states, matching
/// `media_upload_sessions.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaUploadSessionStatus {
    Uploading,
    Completed,
    Failed,
    Expired,
}

impl MediaUploadSessionStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Uploading => "uploading",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Expired => "expired",
        }
    }
}

/// Media processing job states, matching `media_processing_jobs.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaProcessingJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl MediaProcessingJobStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// Durable outbox processing states, matching
/// `outbox_events.processing_status` and `email_delivery_outbox.processing_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxProcessingStatus {
    Pending,
    Processing,
    Processed,
    Failed,
}

impl OutboxProcessingStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Processed => "processed",
            Self::Failed => "failed",
        }
    }
}

/// Device push platform identifiers, matching
/// `device_push_tokens.platform`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePushPlatform {
    Apns,
    Fcm,
}

impl DevicePushPlatform {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Apns => "apns",
            Self::Fcm => "fcm",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "apns" => Some(Self::Apns),
            "fcm" => Some(Self::Fcm),
            _ => None,
        }
    }

    pub fn is_supported(value: &str) -> bool {
        Self::from_db_value(value).is_some()
    }
}

/// Administrative media review actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaReviewAction {
    Quarantine,
    Release,
}

impl MediaReviewAction {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Quarantine => "quarantine",
            Self::Release => "release",
        }
    }

    pub fn from_input(value: &str) -> Option<Self> {
        match value {
            "quarantine" => Some(Self::Quarantine),
            "release" => Some(Self::Release),
            _ => None,
        }
    }
}

/// Resource kinds a media object may be attached to, matching
/// `media_objects.resource_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaResourceType {
    Message,
    ResearchProject,
}

impl MediaResourceType {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::ResearchProject => "research_project",
        }
    }
}

/// Notification categories, matching `notifications.category`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationCategory {
    Message,
    Conversation,
    Research,
    Agent,
    Community,
    System,
}

impl NotificationCategory {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Conversation => "conversation",
            Self::Research => "research",
            Self::Agent => "agent",
            Self::Community => "community",
            Self::System => "system",
        }
    }
}

/// Domain event types published to the transactional outbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationEventType {
    NotificationCreated,
}

impl NotificationEventType {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::NotificationCreated => "notification.created",
        }
    }
}

/// Realtime event kinds published to the realtime hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealtimeEventType {
    NotificationCreated,
}

impl RealtimeEventType {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::NotificationCreated => "notification.created",
        }
    }
}
