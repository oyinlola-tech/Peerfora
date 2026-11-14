ALTER TABLE roles
    ADD COLUMN id UUID NOT NULL DEFAULT gen_random_uuid(),
    ADD COLUMN display_name TEXT NOT NULL DEFAULT '',
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN created_by UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ADD COLUMN deleted_at TIMESTAMPTZ;

CREATE UNIQUE INDEX roles_id_unique_idx ON roles (id);

UPDATE roles
SET display_name = initcap(replace(role_key, '_', ' '));

ALTER TABLE roles
    ADD CONSTRAINT roles_role_key_format_check
    CHECK (role_key ~ '^[a-z][a-z0-9_]{2,63}$');

ALTER TABLE permissions
    ADD COLUMN is_assignable BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN is_sensitive BOOLEAN NOT NULL DEFAULT FALSE;

INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive) VALUES
    ('admin.roles.read', 'Read administrative roles', TRUE, FALSE),
    ('admin.roles.create', 'Create custom administrative roles', TRUE, FALSE),
    ('admin.roles.update', 'Update custom administrative roles', TRUE, FALSE),
    ('admin.roles.delete', 'Delete custom administrative roles', TRUE, FALSE),
    ('admin.roles.assign', 'Assign roles to administrators', TRUE, TRUE),
    ('admin.permissions.read', 'Read the administrative permission catalog', TRUE, FALSE),
    ('admin.permissions.assign', 'Assign recognized permissions to custom roles', TRUE, TRUE)
ON CONFLICT (permission_key) DO UPDATE
SET is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

UPDATE permissions
SET is_assignable = TRUE
WHERE permission_key IN (
    'admin.overview.read',
    'accounts.read',
    'accounts.suspend',
    'communities.read',
    'communities.manage',
    'audit_logs.read',
    'admin.accounts.manage'
);

INSERT INTO role_permissions (role_key, permission_key)
SELECT 'super_administrator', permission_key
FROM permissions
WHERE permission_key LIKE 'admin.roles.%'
   OR permission_key LIKE 'admin.permissions.%'
ON CONFLICT DO NOTHING;

CREATE FUNCTION protect_seeded_administrator_and_super_role() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_TABLE_NAME = 'administrators' THEN
        IF OLD.is_seeded_root AND (TG_OP = 'DELETE' OR NEW.is_seeded_root IS DISTINCT FROM TRUE) THEN
            RAISE EXCEPTION 'the seeded administrator identity is protected';
        END IF;
        RETURN CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END;
    ELSIF TG_TABLE_NAME = 'roles' THEN
        IF OLD.role_key = 'super_administrator' THEN
            RAISE EXCEPTION 'the reserved super administrator role is protected';
        END IF;
    ELSIF TG_TABLE_NAME = 'administrator_roles' THEN
        IF OLD.role_key = 'super_administrator'
            AND OLD.administrator_id = '00000000-0000-0000-0000-000000000001' THEN
            RAISE EXCEPTION 'the seeded administrator role assignment is protected';
        END IF;
    ELSIF TG_TABLE_NAME = 'role_permissions' THEN
        IF OLD.role_key = 'super_administrator' THEN
            RAISE EXCEPTION 'the reserved super administrator permissions are protected';
        END IF;
    END IF;
    RETURN CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END;
END;
$$;

CREATE TRIGGER seeded_administrator_protection
    BEFORE UPDATE OR DELETE ON administrators
    FOR EACH ROW EXECUTE FUNCTION protect_seeded_administrator_and_super_role();

CREATE TRIGGER super_administrator_role_protection
    BEFORE UPDATE OR DELETE ON roles
    FOR EACH ROW EXECUTE FUNCTION protect_seeded_administrator_and_super_role();

CREATE TRIGGER seeded_root_role_assignment_protection
    BEFORE UPDATE OR DELETE ON administrator_roles
    FOR EACH ROW EXECUTE FUNCTION protect_seeded_administrator_and_super_role();

CREATE TRIGGER super_administrator_permissions_protection
    BEFORE UPDATE OR DELETE ON role_permissions
    FOR EACH ROW EXECUTE FUNCTION protect_seeded_administrator_and_super_role();
