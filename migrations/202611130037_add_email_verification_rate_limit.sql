ALTER TABLE authentication_rate_limit_buckets
    DROP CONSTRAINT authentication_rate_limit_buckets_operation_check;

ALTER TABLE authentication_rate_limit_buckets
    ADD CONSTRAINT authentication_rate_limit_buckets_operation_check
    CHECK (operation IN (
        'account_registration',
        'account_login',
        'account_password_reset',
        'account_email_verification',
        'administrator_login',
        'administrator_password_reset',
        'agent_registration'
    ));

CREATE INDEX email_verification_tokens_account_id_idx
    ON email_verification_tokens (account_id);
