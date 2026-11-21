use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationError {
    NotificationInvalid,
    NotificationNotFound,
    NotificationPreferencesInvalid,
    ConversationTypeInvalid,
    ConversationTitleInvalid,
    ConversationMemberInvalid,
    ConversationNotFound,
    ConversationMembershipDenied,
    ConversationBlocked,
    ConversationRestricted,
    ConversationAlreadyMember,
    MessageNotFound,
    MessageInvalid,
    DevicePushInvalid,
    DevicePushNotFound,
    MediaInvalid,
    MediaNotFound,
    MediaNotReady,
    MediaUploadTokenInvalid,
    MediaChecksumMismatch,
    RealtimeSubscriptionInvalid,
}

impl CommunicationError {
    pub fn target_id(&self) -> Option<Uuid> {
        None
    }
}

impl std::fmt::Display for CommunicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NotificationInvalid => "notification is invalid",
            Self::NotificationNotFound => "notification was not found",
            Self::NotificationPreferencesInvalid => "notification preferences are invalid",
            Self::ConversationTypeInvalid => "conversation type is invalid",
            Self::ConversationTitleInvalid => "conversation title is invalid",
            Self::ConversationMemberInvalid => "conversation members are invalid",
            Self::ConversationNotFound => "conversation was not found",
            Self::ConversationMembershipDenied => "conversation membership is denied",
            Self::ConversationBlocked => "conversation access is blocked",
            Self::ConversationRestricted => "conversation is restricted",
            Self::ConversationAlreadyMember => "account is already a conversation member",
            Self::MessageNotFound => "message was not found",
            Self::MessageInvalid => "message is invalid",
            Self::DevicePushInvalid => "device token is invalid",
            Self::DevicePushNotFound => "device token was not found",
            Self::MediaInvalid => "media is invalid",
            Self::MediaNotFound => "media was not found",
            Self::MediaNotReady => "media is not ready",
            Self::MediaUploadTokenInvalid => "media upload token is invalid",
            Self::MediaChecksumMismatch => "media checksum does not match",
            Self::RealtimeSubscriptionInvalid => "realtime subscription is invalid",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CommunicationError {}
