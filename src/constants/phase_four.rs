pub const NOTIFICATIONS_ROUTE: &str = "/api/v1/notifications";
pub const NOTIFICATION_UNREAD_COUNT_ROUTE: &str = "/api/v1/notifications/unread-count";
pub const NOTIFICATION_READ_ROUTE: &str = "/api/v1/notifications/{id}/read";
pub const NOTIFICATION_READ_ALL_ROUTE: &str = "/api/v1/notifications/read-all";
pub const NOTIFICATION_PREFERENCES_ROUTE: &str = "/api/v1/notification-preferences";

pub const CONVERSATIONS_ROUTE: &str = "/api/v1/conversations";
pub const CONVERSATION_ROUTE: &str = "/api/v1/conversations/{id}";
pub const CONVERSATION_MEMBERS_ROUTE: &str = "/api/v1/conversations/{id}/members";
pub const CONVERSATION_MEMBER_ROUTE: &str = "/api/v1/conversations/{id}/members/{account_id}";
pub const CONVERSATION_MESSAGES_ROUTE: &str = "/api/v1/conversations/{id}/messages";

pub const MESSAGE_ROUTE: &str = "/api/v1/messages/{id}";
pub const MESSAGE_READ_ROUTE: &str = "/api/v1/messages/{id}/read";
pub const MESSAGE_REPORT_ROUTE: &str = "/api/v1/messages/{id}/reports";

pub const CONVERSATION_REPORT_ROUTE: &str = "/api/v1/conversations/{id}/reports";
pub const MEDIA_REPORT_ROUTE: &str = "/api/v1/media/{id}/reports";

pub const DEVICE_PUSH_ROUTE: &str = "/api/v1/devices/push";
pub const DEVICE_PUSH_BY_ID_ROUTE: &str = "/api/v1/devices/push/{id}";

pub const MEDIA_UPLOADS_ROUTE: &str = "/api/v1/media/uploads";
pub const MEDIA_UPLOAD_BYTES_ROUTE: &str = "/api/v1/media/uploads/{id}/bytes";
pub const MEDIA_UPLOAD_COMPLETE_ROUTE: &str = "/api/v1/media/uploads/{id}/complete";
pub const MEDIA_LIST_ROUTE: &str = "/api/v1/media";
pub const MEDIA_ROUTE: &str = "/api/v1/media/{id}";
pub const MEDIA_DOWNLOAD_ROUTE: &str = "/api/v1/media/{id}/download";

pub const REALTIME_ROUTE: &str = "/api/v1/realtime";

pub const ADMIN_COMMUNICATION_ROUTE: &str = "/api/v1/admin/communication";
pub const ADMIN_CONVERSATION_RESTRICT_ROUTE: &str =
    "/api/v1/admin/communication/{conversation_id}/restrict";
pub const ADMIN_CONVERSATION_CLOSE_ROUTE: &str =
    "/api/v1/admin/communication/{conversation_id}/close";
pub const ADMIN_MEDIA_ROUTE: &str = "/api/v1/admin/media/{id}/review";
pub const ADMIN_REALTIME_ROUTE: &str = "/api/v1/admin/realtime/{account_id}/revoke";

pub const NOTIFICATION_DEFAULT_PAGE_SIZE: i64 = 20;
pub const NOTIFICATION_MAX_PAGE_SIZE: i64 = 100;
pub const NOTIFICATION_MAX_PAGE_OFFSET: i64 = 10_000;
pub const NOTIFICATION_MAX_TITLE_LENGTH: usize = 200;
pub const NOTIFICATION_MAX_BODY_LENGTH: usize = 2_000;
pub const CONVERSATION_MAX_TITLE_LENGTH: usize = 100;
pub const MESSAGE_MAX_BODY_LENGTH: usize = 10_000;
pub const MESSAGE_DEFAULT_PAGE_SIZE: i64 = 50;
pub const MESSAGE_MAX_PAGE_SIZE: i64 = 100;
pub const MESSAGE_MAX_PAGE_OFFSET: i64 = 10_000;
pub const CONVERSATION_MAX_MEMBERS: i64 = 50;
pub const CONVERSATION_DEFAULT_PAGE_SIZE: i64 = 20;
pub const CONVERSATION_MAX_PAGE_SIZE: i64 = 100;
pub const CONVERSATION_MAX_PAGE_OFFSET: i64 = 10_000;
pub const MEDIA_MAX_UPLOAD_BYTES: i64 = 10 * 1024 * 1024;
pub const MEDIA_DEFAULT_PAGE_SIZE: i64 = 20;
pub const MEDIA_MAX_PAGE_SIZE: i64 = 100;
pub const MEDIA_MAX_PAGE_OFFSET: i64 = 10_000;
pub const MEDIA_MAX_FILENAME_LENGTH: usize = 255;
pub const MEDIA_UPLOAD_SESSION_LIFETIME_SECONDS: i64 = 900;
pub const MEDIA_PROCESSING_INTERVAL_SECONDS: u64 = 30;
pub const MEDIA_PROCESSING_MAX_ATTEMPTS: i32 = 3;
pub const EMAIL_OUTBOX_INTERVAL_SECONDS: u64 = 15;
pub const EMAIL_OUTBOX_MAX_ATTEMPTS: i32 = 5;
pub const REALTIME_MAX_EVENT_BYTES: usize = 8_192;
pub const REALTIME_IDLE_TIMEOUT_SECONDS: u64 = 90;
pub const REALTIME_HEARTBEAT_INTERVAL_SECONDS: u64 = 20;
pub const REALTIME_MAX_SUBSCRIPTIONS: usize = 20;
pub const REALTIME_SSE_PING_TEXT: &str = "ping";
pub const REALTIME_HUB_CHANNEL_CAPACITY: usize = 512;

/// Validation length limits shared by the notification, media, and device
/// push repositories. These mirror the database CHECK constraints.
pub const NOTIFICATION_MAX_CATEGORY_LENGTH: usize = 100;
pub const NOTIFICATION_MAX_RESOURCE_TYPE_LENGTH: usize = 50;
pub const MEDIA_MAX_RESOURCE_TYPE_LENGTH: usize = 50;
pub const MEDIA_MAX_MEDIA_TYPE_LENGTH: usize = 100;
pub const DEVICE_PUSH_MIN_TOKEN_LENGTH: usize = 16;
pub const DEVICE_PUSH_MAX_TOKEN_LENGTH: usize = 512;

/// Seconds a claimed outbox or media-processing row stays leased to a worker
/// before another worker may reclaim it.
pub const OUTBOX_CLAIM_SECONDS: i64 = 60;
pub const MEDIA_PROCESSING_CLAIM_SECONDS: i64 = 60;

/// Fallback subject used when a decrypted email delivery payload has none.
pub const EMAIL_FALLBACK_SUBJECT: &str = "Peerfora notification";

/// Notification resource type used when a notification references a
/// conversation (for example, a new-message notification).
pub const NOTIFICATION_RESOURCE_TYPE_CONVERSATION: &str = "conversation";

/// Transactional-outbox aggregate type for notification events.
pub const OUTBOX_AGGREGATE_TYPE_NOTIFICATION: &str = "notification";

/// Administrative permission required to quarantine or release media objects.
pub const ADMIN_MEDIA_QUARANTINE_PERMISSION: &str = "media.quarantine";

/// Administrative permission required to restrict or close conversations.
pub const ADMIN_MESSAGING_ENFORCEMENT_MANAGE_PERMISSION: &str = "messaging.enforcement.manage";
