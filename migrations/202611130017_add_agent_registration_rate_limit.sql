ALTER TABLE authentication_rate_limit_buckets
    DROP CONSTRAINT authentication_rate_limit_buckets_operation_check;

ALTER TABLE authentication_rate_limit_buckets
    ADD CONSTRAINT authentication_rate_limit_buckets_operation_check
    CHECK (operation IN (
        'account_registration',
        'account_login',
        'account_password_reset',
        'administrator_login',
        'administrator_password_reset',
        'agent_registration'
    ));
