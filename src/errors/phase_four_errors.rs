use axum::http::StatusCode;

use crate::enums::phase_four::CommunicationError;

pub fn get_phase_four_error_response(error: CommunicationError) -> (StatusCode, &'static str) {
    match error {
        CommunicationError::NotificationInvalid => (
            StatusCode::BAD_REQUEST,
            "Notification title, body, category, or resource is invalid",
        ),
        CommunicationError::NotificationNotFound => (
            StatusCode::NOT_FOUND,
            "The requested notification was not found",
        ),
        CommunicationError::NotificationPreferencesInvalid => (
            StatusCode::BAD_REQUEST,
            "Notification preferences are invalid",
        ),
        CommunicationError::ConversationTypeInvalid => {
            (StatusCode::BAD_REQUEST, "The conversation type is invalid")
        }
        CommunicationError::ConversationTitleInvalid => {
            (StatusCode::BAD_REQUEST, "The conversation title is invalid")
        }
        CommunicationError::ConversationMemberInvalid => (
            StatusCode::BAD_REQUEST,
            "The conversation member list is invalid",
        ),
        CommunicationError::ConversationNotFound => (
            StatusCode::NOT_FOUND,
            "The requested conversation was not found",
        ),
        CommunicationError::ConversationMembershipDenied => (
            StatusCode::FORBIDDEN,
            "The account is not an active member of this conversation",
        ),
        CommunicationError::ConversationBlocked => (
            StatusCode::FORBIDDEN,
            "This conversation cannot be accessed or messaged",
        ),
        CommunicationError::ConversationRestricted => (
            StatusCode::CONFLICT,
            "This conversation is restricted and cannot send messages",
        ),
        CommunicationError::ConversationAlreadyMember => (
            StatusCode::CONFLICT,
            "The account is already an active conversation member",
        ),
        CommunicationError::MessageNotFound => {
            (StatusCode::NOT_FOUND, "The requested message was not found")
        }
        CommunicationError::MessageInvalid => {
            (StatusCode::BAD_REQUEST, "The message body is invalid")
        }
        CommunicationError::DevicePushInvalid => (
            StatusCode::BAD_REQUEST,
            "The device token or platform is invalid",
        ),
        CommunicationError::DevicePushNotFound => (
            StatusCode::NOT_FOUND,
            "The requested device token was not found",
        ),
        CommunicationError::MediaInvalid => (
            StatusCode::BAD_REQUEST,
            "The media upload or media resource is invalid",
        ),
        CommunicationError::MediaNotFound => (
            StatusCode::NOT_FOUND,
            "The requested media object was not found",
        ),
        CommunicationError::MediaNotReady => (
            StatusCode::CONFLICT,
            "The media object is not ready for download",
        ),
        CommunicationError::MediaUploadTokenInvalid => (
            StatusCode::FORBIDDEN,
            "The media upload session token is invalid or expired",
        ),
        CommunicationError::MediaChecksumMismatch => (
            StatusCode::CONFLICT,
            "The uploaded media checksum does not match",
        ),
        CommunicationError::RealtimeSubscriptionInvalid => (
            StatusCode::BAD_REQUEST,
            "The realtime subscription is invalid",
        ),
    }
}
