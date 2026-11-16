CREATE TABLE developer_application_credentials (
    id UUID PRIMARY KEY,
    application_id UUID NOT NULL REFERENCES developer_applications(id) ON DELETE RESTRICT,
    token_hash CHAR(64) NOT NULL UNIQUE,
    name TEXT NOT NULL CHECK (length(btrim(name)) BETWEEN 3 AND 80),
    environment TEXT NOT NULL CHECK (environment IN ('sandbox', 'production')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    UNIQUE (id, application_id),
    CHECK (expires_at > created_at),
    CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);

CREATE INDEX developer_application_credentials_application_created_idx
    ON developer_application_credentials (application_id, created_at DESC, id DESC);
CREATE INDEX developer_application_credentials_expiry_idx
    ON developer_application_credentials (expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE developer_credential_scopes (
    credential_id UUID NOT NULL,
    application_id UUID NOT NULL,
    scope_key TEXT NOT NULL,
    PRIMARY KEY (credential_id, scope_key),
    FOREIGN KEY (credential_id, application_id)
        REFERENCES developer_application_credentials(id, application_id) ON DELETE RESTRICT,
    FOREIGN KEY (application_id, scope_key)
        REFERENCES developer_application_requested_scopes(application_id, scope_key) ON DELETE RESTRICT
);

CREATE TABLE developer_credential_rate_limits (
    credential_id UUID PRIMARY KEY
        REFERENCES developer_application_credentials(id) ON DELETE RESTRICT,
    window_started_at TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL CHECK (request_count > 0)
);

INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive) VALUES
    ('developers.applications.suspend', 'Suspend approved developer applications', TRUE, TRUE),
    ('developers.credentials.revoke', 'Revoke developer application credentials', TRUE, TRUE)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO roles (role_key, description, display_name)
VALUES ('developer_administrator', 'Manage developer applications and credentials', 'Developer Administrator')
ON CONFLICT (role_key) DO NOTHING;

INSERT INTO role_permissions (role_key, permission_key) VALUES
    ('super_administrator', 'developers.applications.suspend'),
    ('super_administrator', 'developers.credentials.revoke'),
    ('developer_administrator', 'developers.applications.read'),
    ('developer_administrator', 'developers.applications.review'),
    ('developer_administrator', 'developers.applications.suspend'),
    ('developer_administrator', 'developers.credentials.revoke')
ON CONFLICT DO NOTHING;
