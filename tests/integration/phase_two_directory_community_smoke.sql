DO $database_safety_check$
BEGIN
    IF current_database() !~* '(^|_)test($|_)' THEN
        RAISE EXCEPTION 'this smoke test may run only in a test database';
    END IF;
END
$database_safety_check$;

BEGIN;

DO $phase_two_test$
DECLARE
    owner_id UUID;
    reader_id UUID;
    community_id UUID;
    restricted_community_id UUID;
    subject_id UUID;
BEGIN
    INSERT INTO accounts (email, username, password_hash, eligibility_country_code, age_policy_version, terms_accepted_at)
    VALUES ('phase-two-smoke-owner@example.test', 'phase_two_smoke_owner', 'test-password-hash', 'NG', 'test-policy', now())
    RETURNING id INTO owner_id;

    INSERT INTO accounts (email, username, password_hash, eligibility_country_code, age_policy_version, terms_accepted_at)
    VALUES ('phase-two-smoke-reader@example.test', 'phase_two_smoke_reader', 'test-password-hash', 'CA', 'test-policy', now())
    RETURNING id INTO reader_id;

    SELECT id INTO subject_id FROM academic_subjects WHERE slug = 'data-structures';
    IF subject_id IS NULL THEN
        RAISE EXCEPTION 'seeded Data Structures subject is missing';
    END IF;

    INSERT INTO communities (
        slug, display_name, category, visibility, membership_policy,
        creator_account_id, owner_account_id
    ) VALUES (
        'phase-two-private-smoke', 'Phase Two Private Smoke', 'academic_subject', 'private',
        'approval_required', owner_id, owner_id
    ) RETURNING id INTO community_id;

    INSERT INTO community_subjects (community_id, subject_id)
    VALUES (community_id, subject_id);

    INSERT INTO community_members (
        community_id, account_id, membership_status, community_role, joined_at
    ) VALUES (community_id, owner_id, 'active', 'owner', now());

    INSERT INTO community_audit_events (
        community_id, actor_account_id, event_type, target_account_id
    ) VALUES (community_id, owner_id, 'community_created', NULL);

    INSERT INTO outbox_events (event_type, aggregate_type, aggregate_id, payload)
    VALUES ('community_created', 'community', community_id, '{"visibility":"private"}'::jsonb);

    INSERT INTO communities (
        slug, display_name, category, visibility, membership_policy,
        creator_account_id, owner_account_id
    ) VALUES (
        'phase-two-restricted-smoke', 'Phase Two Restricted Smoke', 'general',
        'restricted', 'approval_required', owner_id, owner_id
    ) RETURNING id INTO restricted_community_id;

    INSERT INTO community_members (
        community_id, account_id, membership_status, community_role, joined_at
    ) VALUES (restricted_community_id, owner_id, 'active', 'owner', now());

    INSERT INTO community_members (
        community_id, account_id, membership_status, community_role
    ) VALUES (restricted_community_id, reader_id, 'requested', 'member');

    IF EXISTS (
        SELECT 1 FROM communities community
        WHERE community.id = community_id
            AND (
                community.visibility IN ('public', 'restricted')
                OR EXISTS (
                    SELECT 1 FROM community_members membership
                    WHERE membership.community_id = community.id
                        AND membership.account_id = reader_id
                        AND membership.membership_status = 'active'
                )
            )
    ) THEN
        RAISE EXCEPTION 'private community leaked through discovery';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM communities community
        WHERE community.id = community_id
            AND EXISTS (
                SELECT 1 FROM community_members membership
                WHERE membership.community_id = community.id
                    AND membership.account_id = owner_id
                    AND membership.membership_status = 'active'
            )
    ) THEN
        RAISE EXCEPTION 'active member could not access private community';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM outbox_events
        WHERE aggregate_id = community_id AND event_type = 'community_created'
    ) THEN
        RAISE EXCEPTION 'community outbox row missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM community_members membership
        WHERE membership.community_id = restricted_community_id
            AND membership.account_id = reader_id
            AND membership.membership_status = 'requested'
    ) THEN
        RAISE EXCEPTION 'approval-required membership request is missing';
    END IF;

    BEGIN
        INSERT INTO communities (
            slug, display_name, category, creator_account_id, owner_account_id
        ) VALUES (
            'phase-two-owner-mismatch-smoke', 'Owner Mismatch Smoke', 'general',
            owner_id, owner_id
        ) RETURNING id INTO community_id;
        INSERT INTO community_members (
            community_id, account_id, membership_status, community_role, joined_at
        ) VALUES (community_id, reader_id, 'active', 'owner', now());
        SET CONSTRAINTS ALL IMMEDIATE;
        RAISE EXCEPTION 'community ownership mismatch was not rejected';
    EXCEPTION WHEN check_violation THEN
        NULL;
    END;
END
$phase_two_test$;

ROLLBACK;
