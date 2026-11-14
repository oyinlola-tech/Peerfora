CREATE TABLE administrator_password_reset_tokens (
    token_hash CHAR(64) PRIMARY KEY,
    administrator_id UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    CHECK (expires_at > created_at),
    CHECK (consumed_at IS NULL OR revoked_at IS NULL)
);

CREATE INDEX administrator_password_reset_tokens_active_idx
    ON administrator_password_reset_tokens (administrator_id, expires_at)
    WHERE consumed_at IS NULL AND revoked_at IS NULL;
