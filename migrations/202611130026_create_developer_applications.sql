CREATE TABLE developer_profiles (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE RESTRICT,
    display_name TEXT NOT NULL CHECK (length(btrim(display_name)) BETWEEN 3 AND 80),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 1000),
    profile_status TEXT NOT NULL DEFAULT 'active' CHECK (profile_status IN ('active', 'suspended')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE developer_application_scopes (
    scope_key TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

INSERT INTO developer_application_scopes (scope_key, description) VALUES
    ('academic.directory.read', 'Read the public academic directory')
ON CONFLICT (scope_key) DO UPDATE
SET description = EXCLUDED.description, is_active = TRUE;

CREATE TABLE developer_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    name TEXT NOT NULL CHECK (length(btrim(name)) BETWEEN 3 AND 100),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 2000),
    application_type TEXT NOT NULL CHECK (application_type IN ('web', 'server', 'agent_integration')),
    application_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (application_status IN ('draft', 'submitted', 'approved', 'rejected', 'suspended', 'revoked')),
    submitted_at TIMESTAMPTZ,
    reviewed_by_administrator_id UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    reviewed_at TIMESTAMPTZ,
    review_reason TEXT CHECK (review_reason IS NULL OR length(btrim(review_reason)) BETWEEN 1 AND 1000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((application_status = 'draft') = (submitted_at IS NULL)),
    CHECK ((application_status IN ('approved', 'rejected', 'suspended', 'revoked')) = (reviewed_at IS NOT NULL)),
    CHECK ((reviewed_at IS NULL) = (reviewed_by_administrator_id IS NULL)),
    CHECK ((reviewed_at IS NULL) = (review_reason IS NULL))
);

CREATE UNIQUE INDEX developer_applications_owner_name_lower_unique
    ON developer_applications (owner_account_id, lower(name));
CREATE INDEX developer_applications_status_created_idx
    ON developer_applications (application_status, created_at DESC, id DESC);
CREATE INDEX developer_applications_owner_created_idx
    ON developer_applications (owner_account_id, created_at DESC, id DESC);

CREATE TABLE developer_application_requested_scopes (
    application_id UUID NOT NULL REFERENCES developer_applications(id) ON DELETE RESTRICT,
    scope_key TEXT NOT NULL REFERENCES developer_application_scopes(scope_key) ON DELETE RESTRICT,
    scope_status TEXT NOT NULL DEFAULT 'requested' CHECK (scope_status IN ('requested', 'approved', 'rejected')),
    PRIMARY KEY (application_id, scope_key)
);

CREATE TRIGGER developer_profiles_maintain_updated_at
BEFORE UPDATE ON developer_profiles
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();

CREATE TRIGGER developer_applications_maintain_updated_at
BEFORE UPDATE ON developer_applications
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();

INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive) VALUES
    ('developers.applications.read', 'Read developer application administration metadata', TRUE, FALSE),
    ('developers.applications.review', 'Review submitted developer applications', TRUE, TRUE)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO roles (role_key, description, display_name)
VALUES ('developer_application_reviewer', 'Review developer application access requests', 'Developer Application Reviewer')
ON CONFLICT (role_key) DO NOTHING;

INSERT INTO role_permissions (role_key, permission_key) VALUES
    ('super_administrator', 'developers.applications.read'),
    ('super_administrator', 'developers.applications.review'),
    ('platform_administrator', 'developers.applications.read'),
    ('developer_application_reviewer', 'developers.applications.read'),
    ('developer_application_reviewer', 'developers.applications.review')
ON CONFLICT DO NOTHING;
