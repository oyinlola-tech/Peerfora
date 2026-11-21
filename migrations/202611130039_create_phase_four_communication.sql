CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK (length(category) > 0 AND length(category) <= 100),
    title TEXT NOT NULL CHECK (length(title) > 0 AND length(title) <= 200),
    body TEXT NOT NULL DEFAULT '' CHECK (length(body) <= 2000),
    resource_type TEXT NOT NULL CHECK (length(resource_type) > 0 AND length(resource_type) <= 50),
    resource_id UUID,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((is_read) = (read_at IS NOT NULL))
);
CREATE INDEX notifications_recipient_unread_idx
    ON notifications (recipient_account_id, is_read, created_at DESC);
CREATE INDEX notifications_recipient_created_idx
    ON notifications (recipient_account_id, created_at DESC, id);

CREATE TABLE notification_preferences (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    in_app_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    push_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    research_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    agent_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    community_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO notification_preferences (account_id) SELECT id FROM accounts ON CONFLICT DO NOTHING;

CREATE FUNCTION create_default_notification_preferences()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO notification_preferences (account_id) VALUES (NEW.id)
    ON CONFLICT (account_id) DO NOTHING;
    RETURN NEW;
END;
$$;
CREATE TRIGGER accounts_create_default_notification_preferences AFTER INSERT ON accounts
FOR EACH ROW EXECUTE FUNCTION create_default_notification_preferences();

CREATE TABLE device_push_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    platform TEXT NOT NULL CHECK (platform IN ('apns', 'fcm')),
    token TEXT NOT NULL UNIQUE CHECK (length(token) >= 16 AND length(token) <= 512),
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX device_push_tokens_account_idx ON device_push_tokens (account_id, active);

CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_type TEXT NOT NULL CHECK (conversation_type IN ('direct', 'group', 'research_project')),
    title TEXT,
    research_project_id UUID REFERENCES research_projects(id) ON DELETE CASCADE,
    owner_account_id UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    lifecycle_status TEXT NOT NULL DEFAULT 'active'
        CHECK (lifecycle_status IN ('active', 'archived', 'restricted', 'closed')),
    created_by_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((conversation_type = 'research_project') = (research_project_id IS NOT NULL)),
    CHECK ((conversation_type <> 'research_project') = (research_project_id IS NULL))
);
CREATE INDEX conversations_owner_status_idx
    ON conversations (owner_account_id, lifecycle_status, updated_at DESC);

CREATE TABLE conversation_members (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    member_role TEXT NOT NULL DEFAULT 'member' CHECK (member_role IN ('member', 'owner')),
    membership_status TEXT NOT NULL DEFAULT 'active'
        CHECK (membership_status IN ('active', 'left', 'removed')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (conversation_id, account_id),
    CHECK ((membership_status = 'active') = (joined_at IS NOT NULL))
);
CREATE INDEX conversation_members_account_idx
    ON conversation_members (account_id, membership_status, updated_at DESC);

CREATE TABLE user_blocks (
    blocker_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    blocked_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker_account_id, blocked_account_id),
    CHECK (blocker_account_id <> blocked_account_id)
);

CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    parent_message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    body TEXT NOT NULL CHECK (length(body) > 0 AND length(body) <= 10000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (parent_message_id IS NULL OR parent_message_id <> id)
);
CREATE INDEX messages_conversation_created_idx
    ON messages (conversation_id, created_at DESC, id);

CREATE TABLE message_read_states (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    read_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (message_id, account_id)
);

CREATE TABLE media_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    resource_type TEXT NOT NULL CHECK (length(resource_type) > 0 AND length(resource_type) <= 50),
    resource_id UUID,
    object_key TEXT NOT NULL UNIQUE CHECK (length(object_key) > 0 AND length(object_key) <= 200),
    original_filename TEXT NOT NULL CHECK (length(original_filename) > 0 AND length(original_filename) <= 255),
    media_type TEXT NOT NULL CHECK (length(media_type) > 0 AND length(media_type) <= 100),
    size_bytes BIGINT NOT NULL CHECK (size_bytes >= 0),
    checksum TEXT NOT NULL CHECK (length(checksum) = 64),
    processing_status TEXT NOT NULL DEFAULT 'requested'
        CHECK (processing_status IN ('requested', 'uploading', 'uploaded', 'quarantined',
            'processing', 'ready', 'rejected', 'failed', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX media_objects_owner_status_idx ON media_objects (owner_account_id, processing_status);
CREATE INDEX media_objects_resource_idx ON media_objects (resource_type, resource_id);

CREATE TABLE media_upload_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_object_id UUID NOT NULL REFERENCES media_objects(id) ON DELETE CASCADE,
    upload_token_hash CHAR(64) NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'uploading'
        CHECK (status IN ('uploading', 'completed', 'failed', 'expired')),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (expires_at > created_at)
);

CREATE TABLE media_processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_object_id UUID NOT NULL REFERENCES media_objects(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    claimed_until TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((status = 'processing') = (claimed_until IS NOT NULL))
);
CREATE INDEX media_processing_jobs_pending_idx
    ON media_processing_jobs (available_at, created_at) WHERE status IN ('pending', 'failed');

CREATE TABLE email_delivery_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient TEXT NOT NULL,
    encrypted_payload TEXT NOT NULL,
    processing_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (processing_status IN ('pending', 'processing', 'processed', 'failed')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    claimed_until TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((processing_status = 'processing') = (claimed_until IS NOT NULL)),
    CHECK ((processing_status = 'processed') = (processed_at IS NOT NULL))
);
CREATE INDEX email_delivery_outbox_pending_idx
    ON email_delivery_outbox (available_at, created_at)
    WHERE processing_status IN ('pending', 'failed');

CREATE TABLE realtime_event_cursors (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    last_event_id UUID NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE moderation_reports
    ADD COLUMN target_conversation_id UUID REFERENCES conversations(id) ON DELETE RESTRICT,
    ADD COLUMN target_message_id UUID REFERENCES messages(id) ON DELETE RESTRICT,
    ADD COLUMN target_media_id UUID REFERENCES media_objects(id) ON DELETE RESTRICT;
ALTER TABLE moderation_reports DROP CONSTRAINT IF EXISTS moderation_reports_check;
ALTER TABLE moderation_reports
    ADD CONSTRAINT moderation_reports_check CHECK (
        num_nonnulls(
            target_account_id, target_community_id, target_post_id, target_comment_id,
            target_research_project_id, target_research_finding_id,
            target_conversation_id, target_message_id, target_media_id
        ) = 1
    );
CREATE UNIQUE INDEX moderation_reports_conversation_unique_idx
    ON moderation_reports (reporter_account_id, target_conversation_id, category)
    WHERE target_conversation_id IS NOT NULL AND status IN ('open', 'assigned');
CREATE UNIQUE INDEX moderation_reports_message_unique_idx
    ON moderation_reports (reporter_account_id, target_message_id, category)
    WHERE target_message_id IS NOT NULL AND status IN ('open', 'assigned');
CREATE UNIQUE INDEX moderation_reports_media_unique_idx
    ON moderation_reports (reporter_account_id, target_media_id, category)
    WHERE target_media_id IS NOT NULL AND status IN ('open', 'assigned');

INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive) VALUES
    ('messaging.abuse.read', 'Read messaging abuse reports', TRUE, TRUE),
    ('messaging.reports.review', 'Review and resolve messaging abuse reports', TRUE, TRUE),
    ('messaging.enforcement.manage', 'Restrict or close conversations', TRUE, TRUE),
    ('notifications.delivery.read', 'Read notification delivery records', TRUE, FALSE),
    ('notifications.delivery.retry', 'Retry failed notification deliveries', TRUE, TRUE),
    ('media.read', 'Read media metadata and moderation records', TRUE, FALSE),
    ('media.review', 'Review quarantined or rejected media', TRUE, TRUE),
    ('media.quarantine', 'Quarantine or release media', TRUE, TRUE),
    ('media.remove', 'Remove media objects', TRUE, TRUE),
    ('realtime.connections.read', 'Read active realtime connection records', TRUE, TRUE),
    ('realtime.sessions.revoke', 'Revoke realtime sessions', TRUE, TRUE)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO role_permissions (role_key, permission_key)
SELECT 'super_administrator', permission_key
FROM permissions
WHERE permission_key LIKE 'messaging.%'
    OR permission_key LIKE 'notifications.%'
    OR permission_key LIKE 'media.%'
    OR permission_key LIKE 'realtime.%'
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_key, permission_key)
VALUES
    ('trust_safety_lead', 'messaging.abuse.read'),
    ('trust_safety_lead', 'messaging.reports.review'),
    ('trust_safety_lead', 'messaging.enforcement.manage'),
    ('trust_safety_lead', 'notifications.delivery.read'),
    ('trust_safety_lead', 'notifications.delivery.retry'),
    ('trust_safety_lead', 'media.read'),
    ('trust_safety_lead', 'media.review'),
    ('trust_safety_lead', 'media.quarantine'),
    ('trust_safety_lead', 'media.remove'),
    ('trust_safety_lead', 'realtime.connections.read'),
    ('trust_safety_lead', 'realtime.sessions.revoke'),
    ('content_moderator', 'messaging.abuse.read'),
    ('content_moderator', 'messaging.reports.review'),
    ('content_moderator', 'media.read'),
    ('content_moderator', 'media.review')
ON CONFLICT DO NOTHING;
