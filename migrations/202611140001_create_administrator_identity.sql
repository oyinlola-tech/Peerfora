CREATE TABLE administrators (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'suspended', 'revoked')),
    failed_login_attempts SMALLINT NOT NULL DEFAULT 0 CHECK (failed_login_attempts BETWEEN 0 AND 5),
    login_locked_until TIMESTAMPTZ,
    mfa_secret_ciphertext TEXT,
    mfa_enabled_at TIMESTAMPTZ,
    last_totp_step BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((mfa_secret_ciphertext IS NULL) = (mfa_enabled_at IS NULL))
);

CREATE UNIQUE INDEX administrators_email_lower_unique_idx ON administrators (lower(email));

CREATE TABLE roles (
    role_key TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    is_system_role BOOLEAN NOT NULL DEFAULT TRUE
);

INSERT INTO roles (role_key, description) VALUES
    ('super_administrator', 'Unrestricted platform administration'),
    ('platform_administrator', 'General platform operations'),
    ('trust_safety_lead', 'Trust and safety operations'),
    ('content_moderator', 'Platform content moderation'),
    ('support_agent', 'Limited user support'),
    ('verification_reviewer', 'Student verification review'),
    ('academic_directory_administrator', 'Academic directory maintenance'),
    ('analytics_viewer', 'Aggregate platform overview access');

CREATE TABLE administrator_roles (
    id UUID PRIMARY KEY,
    administrator_id UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    role_key TEXT NOT NULL REFERENCES roles(role_key) ON DELETE RESTRICT,
    granted_by UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (administrator_id, role_key)
);

CREATE TABLE permissions (
    permission_key TEXT PRIMARY KEY,
    description TEXT NOT NULL
);

CREATE TABLE role_permissions (
    role_key TEXT NOT NULL REFERENCES roles(role_key) ON DELETE RESTRICT,
    permission_key TEXT NOT NULL REFERENCES permissions(permission_key) ON DELETE RESTRICT,
    PRIMARY KEY (role_key, permission_key)
);

CREATE TABLE administrator_sessions (
    id UUID PRIMARY KEY,
    administrator_id UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    token_hash CHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    CHECK (expires_at > created_at)
);

CREATE INDEX administrator_sessions_owner_active_idx
    ON administrator_sessions (administrator_id, created_at DESC)
    WHERE revoked_at IS NULL;
CREATE INDEX administrator_sessions_expiration_idx ON administrator_sessions (expires_at);

CREATE TABLE administrator_mfa_challenges (
    token_hash CHAR(64) PRIMARY KEY,
    administrator_id UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    purpose TEXT NOT NULL CHECK (purpose IN ('enrollment', 'login')),
    encrypted_secret TEXT,
    failed_attempts SMALLINT NOT NULL DEFAULT 0 CHECK (failed_attempts BETWEEN 0 AND 5),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    CHECK (expires_at > created_at),
    CHECK ((purpose = 'enrollment') OR encrypted_secret IS NULL)
);

CREATE INDEX administrator_mfa_challenges_expiration_idx
    ON administrator_mfa_challenges (expires_at);

CREATE TABLE administrative_audit_logs (
    id UUID PRIMARY KEY,
    actor_administrator_id UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id UUID NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('success', 'failure')),
    reason TEXT,
    correlation_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX administrative_audit_logs_actor_time_idx
    ON administrative_audit_logs (actor_administrator_id, created_at DESC);
CREATE INDEX administrative_audit_logs_target_time_idx
    ON administrative_audit_logs (target_type, target_id, created_at DESC);

CREATE FUNCTION prevent_administrative_audit_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'administrative audit records are append-only';
END;
$$;

CREATE TRIGGER administrative_audit_logs_append_only
    BEFORE UPDATE OR DELETE ON administrative_audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_administrative_audit_mutation();

INSERT INTO permissions (permission_key, description) VALUES
    ('admin.overview.read', 'Read administrative overview metrics'),
    ('accounts.read', 'Read account administration data'),
    ('accounts.restrict', 'Apply account restrictions'),
    ('accounts.suspend', 'Suspend and reinstate accounts'),
    ('communities.read', 'Read community administration data'),
    ('communities.manage', 'Manage platform community status and ownership'),
    ('reports.read', 'Review user reports'),
    ('reports.resolve', 'Resolve user reports'),
    ('content.moderate', 'Apply platform content moderation actions'),
    ('verification.read', 'Read student verification requests'),
    ('verification.review', 'Review student verification requests'),
    ('directory.manage', 'Manage academic directory records'),
    ('appeals.review', 'Review enforcement appeals'),
    ('audit_logs.read', 'Read administrative audit records'),
    ('country_policies.manage', 'Manage country eligibility policies'),
    ('admin.roles.manage', 'Manage administrator roles'),
    ('admin.accounts.manage', 'Manage administrator identities');

INSERT INTO role_permissions (role_key, permission_key)
SELECT 'super_administrator', permission_key FROM permissions;

INSERT INTO role_permissions (role_key, permission_key) VALUES
    ('platform_administrator', 'admin.overview.read'),
    ('platform_administrator', 'accounts.read'),
    ('platform_administrator', 'accounts.suspend'),
    ('platform_administrator', 'communities.read'),
    ('platform_administrator', 'communities.manage'),
    ('platform_administrator', 'audit_logs.read'),
    ('trust_safety_lead', 'admin.overview.read'),
    ('trust_safety_lead', 'accounts.read'),
    ('trust_safety_lead', 'accounts.restrict'),
    ('trust_safety_lead', 'accounts.suspend'),
    ('trust_safety_lead', 'communities.read'),
    ('trust_safety_lead', 'reports.read'),
    ('trust_safety_lead', 'reports.resolve'),
    ('trust_safety_lead', 'content.moderate'),
    ('trust_safety_lead', 'appeals.review'),
    ('content_moderator', 'reports.read'),
    ('content_moderator', 'reports.resolve'),
    ('content_moderator', 'content.moderate'),
    ('support_agent', 'accounts.read'),
    ('support_agent', 'reports.read'),
    ('verification_reviewer', 'verification.read'),
    ('verification_reviewer', 'verification.review'),
    ('academic_directory_administrator', 'directory.manage'),
    ('analytics_viewer', 'admin.overview.read');
