INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive)
VALUES
    ('agents.read', 'Read registered agent profiles and review state', TRUE, FALSE),
    ('agents.review', 'Approve or reject agent registrations and requested capabilities', TRUE, TRUE)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO roles (role_key, description, is_system_role, display_name)
VALUES ('agent_reviewer', 'Review agent registration and requested capabilities', TRUE, 'Agent Reviewer')
ON CONFLICT (role_key) DO NOTHING;

INSERT INTO role_permissions (role_key, permission_key)
VALUES
    ('super_administrator', 'agents.read'),
    ('super_administrator', 'agents.review'),
    ('trust_safety_lead', 'agents.read'),
    ('trust_safety_lead', 'agents.review'),
    ('agent_reviewer', 'agents.read'),
    ('agent_reviewer', 'agents.review')
ON CONFLICT DO NOTHING;
