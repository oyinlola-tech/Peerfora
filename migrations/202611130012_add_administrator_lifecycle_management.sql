ALTER TABLE administrators
    DROP CONSTRAINT administrators_status_check,
    ADD CONSTRAINT administrators_status_check
        CHECK (status IN ('pending_activation', 'active', 'suspended', 'revoked', 'deleted')),
    DROP CONSTRAINT administrators_seeded_root_activation_check,
    ADD CONSTRAINT administrators_seeded_root_activation_check
        CHECK (NOT is_seeded_root OR status IN ('pending_activation', 'active')),
    DROP CONSTRAINT administrators_password_hash_state_check,
    ADD CONSTRAINT administrators_password_hash_state_check
        CHECK (
            (is_seeded_root AND status = 'pending_activation' AND password_hash IS NULL)
            OR (status = 'deleted' AND password_hash IS NULL)
            OR password_hash IS NOT NULL
        ),
    ADD COLUMN created_by UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    ADD COLUMN suspended_at TIMESTAMPTZ,
    ADD COLUMN deleted_at TIMESTAMPTZ,
    ADD COLUMN lifecycle_changed_by UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    ADD COLUMN lifecycle_reason TEXT CHECK (
        lifecycle_reason IS NULL OR length(btrim(lifecycle_reason)) BETWEEN 1 AND 500
    );

UPDATE administrators SET suspended_at = updated_at WHERE status = 'suspended';

ALTER TABLE administrators
    ADD CONSTRAINT administrators_suspended_at_state_check
        CHECK ((status = 'suspended') = (suspended_at IS NOT NULL)),
    ADD CONSTRAINT administrators_deleted_at_state_check
        CHECK ((status = 'deleted') = (deleted_at IS NOT NULL));

ALTER TABLE administrator_roles
    DROP CONSTRAINT administrator_roles_administrator_id_role_key_key,
    ADD COLUMN revoked_at TIMESTAMPTZ,
    ADD COLUMN revoked_by UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    ADD COLUMN revocation_reason TEXT CHECK (
        revocation_reason IS NULL OR length(btrim(revocation_reason)) BETWEEN 1 AND 500
    ),
    ADD CONSTRAINT administrator_role_revocation_actor_check
        CHECK ((revoked_at IS NULL) = (revoked_by IS NULL));

CREATE UNIQUE INDEX administrator_roles_active_unique_idx
    ON administrator_roles (administrator_id, role_key)
    WHERE revoked_at IS NULL;

CREATE INDEX administrators_active_super_admin_idx
    ON administrators (id)
    WHERE status = 'active' AND mfa_enabled_at IS NOT NULL;

CREATE OR REPLACE FUNCTION protect_seeded_administrator_and_super_role() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_TABLE_NAME = 'administrators' THEN
        IF OLD.is_seeded_root AND (
            TG_OP = 'DELETE'
            OR NEW.is_seeded_root IS DISTINCT FROM TRUE
            OR NEW.status NOT IN ('pending_activation', 'active')
        ) THEN
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

CREATE OR REPLACE FUNCTION prevent_administrative_audit_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'administrative audit records are append-only';
END;
$$;
