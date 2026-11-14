ALTER TABLE administrators
    ADD COLUMN failed_mfa_attempts SMALLINT NOT NULL DEFAULT 0
        CHECK (failed_mfa_attempts BETWEEN 0 AND 5),
    ADD COLUMN mfa_locked_until TIMESTAMPTZ;
