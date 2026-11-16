INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive)
VALUES (
    'agents.manage',
    'Suspend, reinstate, or revoke approved agent identities',
    TRUE,
    TRUE
)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO role_permissions (role_key, permission_key)
VALUES
    ('super_administrator', 'agents.manage'),
    ('trust_safety_lead', 'agents.manage')
ON CONFLICT DO NOTHING;
