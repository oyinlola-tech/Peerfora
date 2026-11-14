INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive)
VALUES (
    'accounts.sessions.revoke',
    'Revoke all active sessions for an account',
    TRUE,
    TRUE
)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO role_permissions (role_key, permission_key)
VALUES
    ('super_administrator', 'accounts.sessions.revoke'),
    ('platform_administrator', 'accounts.sessions.revoke'),
    ('trust_safety_lead', 'accounts.sessions.revoke')
ON CONFLICT DO NOTHING;
