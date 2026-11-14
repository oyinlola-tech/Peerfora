CREATE TABLE administrator_invitations (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL,
    token_hash CHAR(64) NOT NULL UNIQUE,
    role_key TEXT NOT NULL REFERENCES roles(role_key) ON DELETE RESTRICT,
    invited_by UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    CHECK (expires_at > created_at),
    CHECK (consumed_at IS NULL OR revoked_at IS NULL)
);

CREATE UNIQUE INDEX administrator_invitations_pending_email_idx
    ON administrator_invitations (lower(email))
    WHERE consumed_at IS NULL AND revoked_at IS NULL;

CREATE INDEX administrator_invitations_expiration_idx
    ON administrator_invitations (expires_at)
    WHERE consumed_at IS NULL AND revoked_at IS NULL;
