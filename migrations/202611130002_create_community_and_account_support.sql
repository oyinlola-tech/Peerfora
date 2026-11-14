CREATE TABLE communities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL,
    display_name TEXT NOT NULL CHECK (length(display_name) BETWEEN 3 AND 80),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 2000),
    category TEXT NOT NULL CHECK (category IN (
        'academic_subject', 'institution', 'institution_course', 'academic_discipline',
        'study_group', 'scholarship', 'internship', 'career', 'student_life', 'general', 'other'
    )),
    institution_id UUID REFERENCES institutions(id) ON DELETE RESTRICT,
    course_id UUID,
    discipline_id UUID REFERENCES academic_disciplines(id) ON DELETE RESTRICT,
    creator_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    owner_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    visibility TEXT NOT NULL DEFAULT 'public' CHECK (visibility IN ('public', 'restricted', 'private')),
    lifecycle_status TEXT NOT NULL DEFAULT 'active' CHECK (lifecycle_status IN ('active', 'archived')),
    posting_policy TEXT NOT NULL DEFAULT 'members' CHECK (posting_policy IN ('members', 'approved_members')),
    membership_policy TEXT NOT NULL DEFAULT 'open' CHECK (membership_policy IN ('open', 'approval_required')),
    primary_language_tag TEXT NOT NULL DEFAULT 'en',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (course_id, institution_id)
        REFERENCES institution_courses(id, institution_id) ON DELETE RESTRICT,
    CHECK (course_id IS NULL OR institution_id IS NOT NULL),
    CHECK (category <> 'institution' OR institution_id IS NOT NULL),
    CHECK (category <> 'institution_course' OR course_id IS NOT NULL),
    CHECK (category <> 'academic_discipline' OR discipline_id IS NOT NULL)
);
CREATE UNIQUE INDEX communities_slug_lower_unique ON communities (lower(slug));
CREATE INDEX communities_public_discovery_idx ON communities (category, created_at DESC)
    WHERE visibility IN ('public', 'restricted') AND lifecycle_status = 'active';
CREATE INDEX communities_institution_idx ON communities (institution_id) WHERE institution_id IS NOT NULL;
CREATE INDEX communities_discipline_idx ON communities (discipline_id) WHERE discipline_id IS NOT NULL;
CREATE TABLE community_subjects (
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (community_id, subject_id)
);
CREATE INDEX community_subjects_subject_idx ON community_subjects (subject_id, community_id);
CREATE TABLE community_members (
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    membership_status TEXT NOT NULL CHECK (membership_status IN (
        'requested', 'invited', 'active', 'rejected', 'left', 'removed', 'banned'
    )),
    community_role TEXT NOT NULL DEFAULT 'member' CHECK (community_role IN ('member', 'moderator', 'owner')),
    joined_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (community_id, account_id),
    CHECK (membership_status <> 'active' OR joined_at IS NOT NULL),
    CHECK (membership_status NOT IN ('requested', 'invited', 'rejected') OR joined_at IS NULL)
);
CREATE UNIQUE INDEX community_one_active_owner_idx ON community_members (community_id)
    WHERE membership_status = 'active' AND community_role = 'owner';
CREATE INDEX community_members_account_status_idx
    ON community_members (account_id, membership_status, updated_at DESC);
CREATE INDEX community_members_active_role_idx ON community_members (community_id, community_role)
    WHERE membership_status = 'active';
CREATE TABLE community_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 100),
    description TEXT NOT NULL CHECK (length(description) BETWEEN 1 AND 2000),
    display_order SMALLINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX community_rules_order_idx ON community_rules (community_id, display_order, created_at)
    WHERE is_active;
CREATE TABLE community_flairs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 40),
    flair_type TEXT NOT NULL CHECK (flair_type IN ('post', 'member')),
    is_assignable_by_member BOOLEAN NOT NULL DEFAULT TRUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (community_id, flair_type, display_name)
);
CREATE TABLE community_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE RESTRICT,
    actor_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    event_type TEXT NOT NULL,
    target_account_id UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX community_audit_events_community_created_idx
    ON community_audit_events (community_id, created_at DESC);

CREATE FUNCTION enforce_community_owner_membership()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    affected_community_id UUID;
    owner_id UUID;
BEGIN
    IF TG_TABLE_NAME = 'communities' THEN
        affected_community_id := COALESCE(NEW.id, OLD.id);
    ELSE
        affected_community_id := COALESCE(NEW.community_id, OLD.community_id);
    END IF;
    SELECT owner_account_id INTO owner_id FROM communities WHERE id = affected_community_id;
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM community_members membership
        JOIN accounts owner_account ON owner_account.id = membership.account_id
        WHERE membership.community_id = affected_community_id
            AND membership.account_id = owner_id AND membership.membership_status = 'active'
            AND membership.community_role = 'owner' AND owner_account.account_status = 'active'
    ) OR EXISTS (
        SELECT 1 FROM community_members
        WHERE community_id = affected_community_id AND membership_status = 'active'
            AND community_role = 'owner' AND account_id <> owner_id
    ) THEN
        RAISE EXCEPTION 'community owner and active owner membership must match'
            USING ERRCODE = '23514';
    END IF;
    IF TG_TABLE_NAME = 'communities' OR TG_TABLE_NAME = 'community_subjects' THEN
        IF EXISTS (SELECT 1 FROM communities WHERE id = affected_community_id
            AND category = 'academic_subject')
            AND NOT EXISTS (SELECT 1 FROM community_subjects
                WHERE community_id = affected_community_id) THEN
            RAISE EXCEPTION 'academic subject communities require at least one subject'
                USING ERRCODE = '23514';
        END IF;
    END IF;
    RETURN NULL;
END;
$$;
CREATE CONSTRAINT TRIGGER communities_owner_membership_consistency
AFTER INSERT OR UPDATE OR DELETE ON communities DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION enforce_community_owner_membership();
CREATE CONSTRAINT TRIGGER community_members_owner_consistency
AFTER INSERT OR UPDATE OR DELETE ON community_members DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION enforce_community_owner_membership();
CREATE CONSTRAINT TRIGGER community_subjects_category_consistency
AFTER INSERT OR UPDATE OR DELETE ON community_subjects DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION enforce_community_owner_membership();

CREATE TABLE user_profiles (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    biography TEXT NOT NULL DEFAULT '' CHECK (length(biography) <= 500),
    avatar_reference TEXT,
    banner_reference TEXT,
    academic_level_id TEXT REFERENCES academic_levels(id) ON DELETE SET NULL,
    institution_id UUID REFERENCES institutions(id) ON DELETE SET NULL,
    discipline_id UUID REFERENCES academic_disciplines(id) ON DELETE SET NULL,
    profile_visibility TEXT NOT NULL DEFAULT 'public' CHECK (profile_visibility IN ('public', 'private')),
    show_academic_affiliation BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE account_preferences (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    language_tag TEXT NOT NULL DEFAULT 'en',
    country_code CHAR(2) NOT NULL CHECK (country_code = upper(country_code)),
    notification_preferences JSONB NOT NULL DEFAULT '{}'::jsonb,
    content_preferences JSONB NOT NULL DEFAULT '{}'::jsonb,
    personalization_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE account_preferences ADD CONSTRAINT account_preferences_country_code_fk
    FOREIGN KEY (country_code) REFERENCES countries(country_code) NOT VALID;
INSERT INTO user_profiles (account_id) SELECT id FROM accounts;
CREATE FUNCTION create_default_account_profile_and_preferences()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO user_profiles (account_id) VALUES (NEW.id);
    INSERT INTO account_preferences (account_id, country_code)
        VALUES (NEW.id, NEW.eligibility_country_code);
    RETURN NEW;
END;
$$;
CREATE TRIGGER accounts_create_default_profile_and_preferences AFTER INSERT ON accounts
FOR EACH ROW EXECUTE FUNCTION create_default_account_profile_and_preferences();

CREATE TABLE outbox_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    schema_version SMALLINT NOT NULL DEFAULT 1 CHECK (schema_version > 0),
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload JSONB NOT NULL,
    processing_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (processing_status IN ('pending', 'processing', 'processed', 'failed')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    claimed_by TEXT,
    claim_token UUID,
    claimed_until TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    last_error TEXT,
    CHECK ((processing_status = 'processing' AND claimed_by IS NOT NULL
        AND claim_token IS NOT NULL AND claimed_until IS NOT NULL)
        OR (processing_status <> 'processing' AND claimed_by IS NULL
        AND claim_token IS NULL AND claimed_until IS NULL)),
    CHECK ((processing_status = 'processed') = (processed_at IS NOT NULL))
);
CREATE INDEX outbox_events_pending_idx ON outbox_events (available_at, occurred_at, id)
    WHERE processing_status IN ('pending', 'failed');
CREATE INDEX outbox_events_expired_claim_idx ON outbox_events (claimed_until)
    WHERE processing_status = 'processing';

CREATE TABLE account_eligibility_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    policy_version TEXT NOT NULL,
    assessment_method TEXT NOT NULL CHECK (assessment_method IN ('self_declared', 'age_assurance')),
    eligible BOOLEAN NOT NULL,
    assessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX account_eligibility_assessments_account_time_idx
    ON account_eligibility_assessments (account_id, assessed_at DESC);
INSERT INTO account_eligibility_assessments (account_id, policy_version, assessment_method, eligible)
SELECT id, age_policy_version, 'self_declared', TRUE FROM accounts;
CREATE FUNCTION record_initial_account_eligibility_assessment()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO account_eligibility_assessments (account_id, policy_version, assessment_method, eligible)
    VALUES (NEW.id, NEW.age_policy_version, 'self_declared', TRUE);
    RETURN NEW;
END;
$$;
CREATE TRIGGER accounts_record_initial_eligibility_assessment AFTER INSERT ON accounts
FOR EACH ROW EXECUTE FUNCTION record_initial_account_eligibility_assessment();

CREATE FUNCTION require_account_authentication_method()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    affected_account_id UUID;
    stored_password_hash TEXT;
BEGIN
    IF TG_TABLE_NAME = 'accounts' THEN
        affected_account_id := COALESCE(NEW.id, OLD.id);
    ELSE
        affected_account_id := COALESCE(NEW.account_id, OLD.account_id);
    END IF;
    SELECT password_hash INTO stored_password_hash FROM accounts WHERE id = affected_account_id;
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;
    IF stored_password_hash IS NULL AND NOT EXISTS (
        SELECT 1 FROM account_social_identities WHERE account_id = affected_account_id
    ) THEN
        RAISE EXCEPTION 'account must have a password or social identity'
            USING ERRCODE = '23514';
    END IF;
    RETURN NULL;
END;
$$;
CREATE CONSTRAINT TRIGGER accounts_require_authentication_method
AFTER INSERT OR UPDATE OR DELETE ON accounts DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION require_account_authentication_method();
CREATE CONSTRAINT TRIGGER social_identity_requires_account_authentication_method
AFTER INSERT OR UPDATE OR DELETE ON account_social_identities DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION require_account_authentication_method();

CREATE INDEX email_verification_tokens_expiry_idx ON email_verification_tokens (expires_at);
CREATE INDEX password_reset_tokens_expiry_idx ON password_reset_tokens (expires_at);
CREATE INDEX account_sessions_expiry_idx ON account_sessions (expires_at);
ALTER TABLE account_sessions
    ADD COLUMN last_seen_at TIMESTAMPTZ,
    ADD COLUMN user_agent TEXT,
    ADD COLUMN remote_address INET;

CREATE TRIGGER academic_disciplines_maintain_updated_at BEFORE UPDATE ON academic_disciplines
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER academic_subjects_maintain_updated_at BEFORE UPDATE ON academic_subjects
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER institutions_maintain_updated_at BEFORE UPDATE ON institutions
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER institution_courses_maintain_updated_at BEFORE UPDATE ON institution_courses
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER communities_maintain_updated_at BEFORE UPDATE ON communities
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER community_members_maintain_updated_at BEFORE UPDATE ON community_members
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER community_rules_maintain_updated_at BEFORE UPDATE ON community_rules
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER user_profiles_maintain_updated_at BEFORE UPDATE ON user_profiles
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
CREATE TRIGGER account_preferences_maintain_updated_at BEFORE UPDATE ON account_preferences
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
