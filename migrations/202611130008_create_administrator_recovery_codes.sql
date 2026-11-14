CREATE TABLE administrator_recovery_codes (
    id UUID PRIMARY KEY,
    administrator_id UUID NOT NULL REFERENCES administrators(id) ON DELETE RESTRICT,
    token_hash CHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    used_at TIMESTAMPTZ,
    UNIQUE (administrator_id, token_hash)
);

CREATE INDEX administrator_recovery_codes_unused_idx
    ON administrator_recovery_codes (administrator_id, created_at)
    WHERE used_at IS NULL;
