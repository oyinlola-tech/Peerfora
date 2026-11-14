ALTER TABLE administrator_recovery_codes
    ADD COLUMN revoked_at TIMESTAMPTZ,
    ADD CONSTRAINT administrator_recovery_code_terminal_state_check
        CHECK (used_at IS NULL OR revoked_at IS NULL);

DROP INDEX administrator_recovery_codes_unused_idx;

CREATE INDEX administrator_recovery_codes_unused_idx
    ON administrator_recovery_codes (administrator_id, created_at)
    WHERE used_at IS NULL AND revoked_at IS NULL;
