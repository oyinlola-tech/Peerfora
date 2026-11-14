CREATE INDEX accounts_admin_email_prefix_idx
    ON accounts (lower(email) text_pattern_ops);

CREATE INDEX accounts_admin_username_prefix_idx
    ON accounts (lower(username) text_pattern_ops);

CREATE INDEX accounts_admin_status_country_created_idx
    ON accounts (account_status, eligibility_country_code, created_at DESC, id);

CREATE INDEX user_profiles_admin_academic_level_idx
    ON user_profiles (academic_level_id, account_id)
    WHERE academic_level_id IS NOT NULL;
