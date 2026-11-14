ALTER TABLE administrators
    ALTER COLUMN password_hash DROP NOT NULL,
    ADD COLUMN is_seeded_root BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE administrators
    DROP CONSTRAINT administrators_status_check,
    ADD CONSTRAINT administrators_status_check
        CHECK (status IN ('pending_activation', 'active', 'suspended', 'revoked')),
    ADD CONSTRAINT administrators_seeded_root_activation_check
        CHECK (NOT is_seeded_root OR status IN ('pending_activation', 'active', 'suspended', 'revoked')),
    ADD CONSTRAINT administrators_password_hash_state_check
        CHECK (
            (is_seeded_root AND status = 'pending_activation' AND password_hash IS NULL)
            OR (password_hash IS NOT NULL)
        );

CREATE UNIQUE INDEX administrators_single_seeded_root_idx
    ON administrators (is_seeded_root)
    WHERE is_seeded_root;

INSERT INTO administrators (id, email, password_hash, status, is_seeded_root)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'root@peerfora.invalid',
    NULL,
    'pending_activation',
    TRUE
);

INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
VALUES (
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000001',
    'super_administrator',
    '00000000-0000-0000-0000-000000000001'
);
