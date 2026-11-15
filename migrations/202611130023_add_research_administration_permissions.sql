INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive) VALUES
    ('research.projects.read', 'Read approved research project administration metadata', TRUE, TRUE),
    ('research.projects.suspend', 'Suspend active research projects', TRUE, TRUE),
    ('research.projects.restore', 'Restore suspended research projects', TRUE, TRUE)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO role_permissions (role_key, permission_key)
SELECT 'super_administrator', permission_key
FROM permissions WHERE permission_key LIKE 'research.projects.%'
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_key, permission_key) VALUES
    ('platform_administrator', 'research.projects.read'),
    ('trust_safety_lead', 'research.projects.read'),
    ('trust_safety_lead', 'research.projects.suspend'),
    ('trust_safety_lead', 'research.projects.restore')
ON CONFLICT DO NOTHING;

INSERT INTO roles (role_key, description, display_name)
VALUES (
    'research_integrity_reviewer',
    'Review research integrity and permitted project metadata',
    'Research Integrity Reviewer'
)
ON CONFLICT (role_key) DO NOTHING;

INSERT INTO role_permissions (role_key, permission_key)
VALUES ('research_integrity_reviewer', 'research.projects.read')
ON CONFLICT DO NOTHING;
