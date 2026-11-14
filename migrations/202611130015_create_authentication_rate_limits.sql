CREATE TABLE authentication_rate_limit_buckets (
    operation TEXT NOT NULL CHECK (operation IN (
        'account_registration',
        'account_login',
        'account_password_reset',
        'administrator_login',
        'administrator_password_reset'
    )),
    subject_hash CHAR(64) NOT NULL,
    window_started_at TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL CHECK (request_count > 0),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (operation, subject_hash)
);

CREATE INDEX authentication_rate_limit_buckets_expiry_idx
    ON authentication_rate_limit_buckets (expires_at);
