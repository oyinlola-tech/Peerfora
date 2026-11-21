use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};

use crate::{
    constants::phase_four::{
        CONVERSATION_MEMBER_ROUTE, CONVERSATION_MEMBERS_ROUTE, CONVERSATION_MESSAGES_ROUTE,
        CONVERSATION_REPORT_ROUTE, CONVERSATION_ROUTE, CONVERSATIONS_ROUTE,
        DEVICE_PUSH_BY_ID_ROUTE, DEVICE_PUSH_ROUTE, MESSAGE_READ_ROUTE, MESSAGE_REPORT_ROUTE,
        MESSAGE_ROUTE, NOTIFICATION_PREFERENCES_ROUTE, NOTIFICATION_READ_ALL_ROUTE,
        NOTIFICATION_READ_ROUTE, NOTIFICATION_UNREAD_COUNT_ROUTE, NOTIFICATIONS_ROUTE,
    },
    controllers::communication_controller::{conversation_handlers, notification_handlers},
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            NOTIFICATIONS_ROUTE,
            get(notification_handlers::list_notifications),
        )
        .route(
            NOTIFICATION_UNREAD_COUNT_ROUTE,
            get(notification_handlers::unread_notification_count),
        )
        .route(
            NOTIFICATION_READ_ALL_ROUTE,
            post(notification_handlers::read_all_notifications),
        )
        .route(
            NOTIFICATION_READ_ROUTE,
            patch(notification_handlers::read_notification),
        )
        .route(
            NOTIFICATION_PREFERENCES_ROUTE,
            get(notification_handlers::get_notification_preferences)
                .put(notification_handlers::update_notification_preferences),
        )
        .route(
            CONVERSATIONS_ROUTE,
            get(conversation_handlers::list_conversations)
                .post(conversation_handlers::create_conversation),
        )
        .route(
            CONVERSATION_ROUTE,
            get(conversation_handlers::get_conversation)
                .patch(conversation_handlers::update_conversation),
        )
        .route(
            CONVERSATION_MEMBERS_ROUTE,
            get(conversation_handlers::list_conversation_members)
                .post(conversation_handlers::add_conversation_members),
        )
        .route(
            CONVERSATION_MEMBER_ROUTE,
            delete(conversation_handlers::remove_conversation_member),
        )
        .route(
            CONVERSATION_MESSAGES_ROUTE,
            get(conversation_handlers::list_conversation_messages)
                .post(conversation_handlers::send_conversation_message),
        )
        .route(MESSAGE_ROUTE, get(conversation_handlers::get_message))
        .route(
            MESSAGE_READ_ROUTE,
            patch(conversation_handlers::mark_message_read),
        )
        .route(
            CONVERSATION_REPORT_ROUTE,
            post(conversation_handlers::report_conversation),
        )
        .route(
            MESSAGE_REPORT_ROUTE,
            post(conversation_handlers::report_message),
        )
        .route(
            DEVICE_PUSH_ROUTE,
            get(notification_handlers::list_device_push_tokens)
                .post(notification_handlers::register_device_push_token),
        )
        .route(
            DEVICE_PUSH_BY_ID_ROUTE,
            get(notification_handlers::get_device_push_token)
                .put(notification_handlers::update_device_push_token),
        )
}
