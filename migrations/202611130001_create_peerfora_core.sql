CREATE TABLE countries (
    country_code CHAR(2) PRIMARY KEY CHECK (country_code ~ '^[A-Z]{2}$'),
    alpha_3 CHAR(3) NOT NULL UNIQUE CHECK (alpha_3 ~ '^[A-Z]{3}$'),
    display_name TEXT NOT NULL CHECK (length(btrim(display_name)) > 0),
    official_name TEXT,
    source TEXT NOT NULL,
    source_version TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE country_name_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    country_code CHAR(2) NOT NULL REFERENCES countries(country_code) ON DELETE RESTRICT,
    display_name TEXT NOT NULL CHECK (length(btrim(display_name)) > 0),
    language_tag TEXT NOT NULL DEFAULT 'und',
    source TEXT NOT NULL,
    source_version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (country_code, display_name, language_tag)
);
CREATE INDEX country_name_aliases_name_idx ON country_name_aliases (lower(display_name));

ALTER TABLE accounts
    ADD CONSTRAINT accounts_country_code_fk
    FOREIGN KEY (eligibility_country_code) REFERENCES countries(country_code) NOT VALID,
    ADD COLUMN account_status TEXT NOT NULL DEFAULT 'active'
        CHECK (account_status IN ('active', 'suspended', 'pending_deletion', 'deleted'));
ALTER TABLE accounts DROP CONSTRAINT accounts_email_key;
CREATE UNIQUE INDEX accounts_email_lower_unique ON accounts (lower(email));
CREATE FUNCTION maintain_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;
CREATE TRIGGER accounts_maintain_updated_at BEFORE UPDATE ON accounts
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();

CREATE TABLE academic_levels (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    display_order SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE academic_level_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_level_id TEXT NOT NULL REFERENCES academic_levels(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (length(btrim(alias)) > 0),
    language_tag TEXT NOT NULL DEFAULT 'und',
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (academic_level_id, alias, language_tag)
);
CREATE UNIQUE INDEX academic_level_aliases_alias_locale_unique
    ON academic_level_aliases (lower(alias), language_tag);
CREATE TABLE academic_disciplines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE academic_subjects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    discipline_id UUID REFERENCES academic_disciplines(id) ON DELETE SET NULL,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX academic_subjects_discipline_idx ON academic_subjects (discipline_id)
    WHERE discipline_id IS NOT NULL;
CREATE TABLE academic_subject_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (length(btrim(alias)) > 0),
    language_tag TEXT NOT NULL DEFAULT 'und',
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (subject_id, alias, language_tag)
);
CREATE INDEX academic_subject_aliases_alias_lower_idx ON academic_subject_aliases (lower(alias));
CREATE TABLE academic_qualification_types (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    academic_level_id TEXT NOT NULL REFERENCES academic_levels(id) ON DELETE RESTRICT,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE institution_types (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE institutions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    identifier_namespace TEXT,
    official_identifier TEXT,
    country_code CHAR(2) REFERENCES countries(country_code) ON DELETE RESTRICT,
    region_name TEXT,
    institution_type TEXT NOT NULL REFERENCES institution_types(id) ON DELETE RESTRICT,
    official_website TEXT,
    verification_status TEXT NOT NULL DEFAULT 'unverified'
        CHECK (verification_status IN ('unverified', 'pending', 'verified', 'rejected', 'inactive')),
    record_source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((official_identifier IS NULL) = (identifier_namespace IS NULL))
);
CREATE UNIQUE INDEX institutions_namespace_identifier_unique
    ON institutions (COALESCE(country_code::text, ''), identifier_namespace, official_identifier)
    WHERE official_identifier IS NOT NULL;
CREATE INDEX institutions_country_type_idx ON institutions (country_code, institution_type)
    WHERE is_active;
CREATE TABLE institution_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id UUID NOT NULL REFERENCES institutions(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (length(btrim(alias)) > 0),
    record_source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (institution_id, alias)
);
CREATE INDEX institution_aliases_alias_lower_idx ON institution_aliases (lower(alias));
CREATE TABLE directory_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suggestion_type TEXT NOT NULL CHECK (suggestion_type IN (
        'academic_level', 'discipline', 'subject', 'qualification_type',
        'institution', 'institution_type', 'other'
    )),
    proposed_name TEXT NOT NULL CHECK (length(btrim(proposed_name)) BETWEEN 2 AND 120),
    parent_name TEXT,
    country_code CHAR(2) REFERENCES countries(country_code) ON DELETE RESTRICT,
    official_website TEXT,
    details TEXT CHECK (details IS NULL OR length(details) <= 2000),
    submitted_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN ('pending', 'accepted', 'rejected', 'duplicate')),
    reviewed_by UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    reviewed_at TIMESTAMPTZ,
    review_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (suggestion_type <> 'institution' OR country_code IS NOT NULL),
    CHECK ((review_status = 'pending' AND reviewed_by IS NULL AND reviewed_at IS NULL)
        OR (review_status <> 'pending' AND reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL))
);
CREATE INDEX directory_suggestions_pending_created_idx
    ON directory_suggestions (created_at DESC) WHERE review_status = 'pending';
CREATE TABLE institution_courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id UUID NOT NULL REFERENCES institutions(id) ON DELETE RESTRICT,
    course_code TEXT NOT NULL CHECK (length(btrim(course_code)) > 0),
    display_name TEXT NOT NULL,
    description TEXT,
    department_name TEXT,
    academic_level_id TEXT REFERENCES academic_levels(id) ON DELETE SET NULL,
    credit_value NUMERIC(6, 2) CHECK (credit_value IS NULL OR credit_value >= 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    record_source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (institution_id, course_code),
    UNIQUE (id, institution_id)
);
CREATE TABLE course_subjects (
    course_id UUID NOT NULL REFERENCES institution_courses(id) ON DELETE CASCADE,
    subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE RESTRICT,
    mapping_source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (course_id, subject_id)
);
CREATE INDEX course_subjects_subject_idx ON course_subjects (subject_id, course_id);

INSERT INTO academic_levels (id, display_name, source, display_order) VALUES
    ('secondary_school', 'Secondary School', 'peerfora_starter_directory', 10),
    ('foundation', 'Foundation', 'peerfora_starter_directory', 20),
    ('vocational', 'Vocational', 'peerfora_starter_directory', 30),
    ('technical', 'Technical', 'peerfora_starter_directory', 40),
    ('diploma', 'Diploma', 'peerfora_starter_directory', 50),
    ('associate_degree', 'Associate Degree', 'peerfora_starter_directory', 60),
    ('undergraduate', 'Undergraduate', 'peerfora_starter_directory', 70),
    ('masters', 'Master''s', 'peerfora_starter_directory', 80),
    ('doctoral', 'Doctoral', 'peerfora_starter_directory', 90),
    ('postdoctoral', 'Postdoctoral', 'peerfora_starter_directory', 100),
    ('professional_qualification', 'Professional Qualification', 'peerfora_starter_directory', 110),
    ('independent_learner', 'Independent Learner', 'peerfora_starter_directory', 120);
INSERT INTO academic_level_aliases (academic_level_id, alias, source)
VALUES ('secondary_school', 'high_school', 'peerfora_starter_directory');
INSERT INTO academic_disciplines (slug, display_name, source) VALUES
    ('computer-science', 'Computer Science', 'peerfora_starter_directory'),
    ('engineering', 'Engineering', 'peerfora_starter_directory'),
    ('medicine', 'Medicine', 'peerfora_starter_directory'),
    ('law', 'Law', 'peerfora_starter_directory'),
    ('mathematics', 'Mathematics', 'peerfora_starter_directory'),
    ('business', 'Business', 'peerfora_starter_directory'),
    ('arts', 'Arts', 'peerfora_starter_directory'),
    ('social-sciences', 'Social Sciences', 'peerfora_starter_directory'),
    ('natural-sciences', 'Natural Sciences', 'peerfora_starter_directory');
INSERT INTO academic_subjects (slug, display_name, discipline_id, source)
SELECT values_row.slug, values_row.display_name, discipline.id, 'peerfora_starter_directory'
FROM (VALUES
    ('data-structures', 'Data Structures', 'computer-science'),
    ('algorithms', 'Algorithms', 'computer-science'),
    ('machine-learning', 'Machine Learning', 'computer-science'),
    ('organic-chemistry', 'Organic Chemistry', 'natural-sciences'),
    ('calculus', 'Calculus', 'mathematics'),
    ('linear-algebra', 'Linear Algebra', 'mathematics'),
    ('human-anatomy', 'Human Anatomy', 'medicine'),
    ('constitutional-law', 'Constitutional Law', 'law'),
    ('research-methodology', 'Research Methodology', 'social-sciences')
) AS values_row(slug, display_name, discipline_slug)
JOIN academic_disciplines discipline ON discipline.slug = values_row.discipline_slug;
INSERT INTO academic_qualification_types (id, display_name, academic_level_id, source) VALUES
    ('bachelor_of_arts', 'Bachelor of Arts', 'undergraduate', 'peerfora_starter_directory'),
    ('bachelor_of_science', 'Bachelor of Science', 'undergraduate', 'peerfora_starter_directory'),
    ('higher_national_diploma', 'Higher National Diploma', 'diploma', 'peerfora_starter_directory'),
    ('master_of_arts', 'Master of Arts', 'masters', 'peerfora_starter_directory'),
    ('master_of_science', 'Master of Science', 'masters', 'peerfora_starter_directory'),
    ('master_of_business_administration', 'Master of Business Administration', 'masters', 'peerfora_starter_directory'),
    ('doctor_of_philosophy', 'Doctor of Philosophy', 'doctoral', 'peerfora_starter_directory'),
    ('doctor_of_education', 'Doctor of Education', 'doctoral', 'peerfora_starter_directory');
INSERT INTO institution_types (id, display_name, source) VALUES
    ('secondary_school', 'Secondary School', 'peerfora_starter_directory'),
    ('university', 'University', 'peerfora_starter_directory'),
    ('college', 'College', 'peerfora_starter_directory'),
    ('community_college', 'Community College', 'peerfora_starter_directory'),
    ('polytechnic', 'Polytechnic', 'peerfora_starter_directory'),
    ('technical', 'Technical Institution', 'peerfora_starter_directory'),
    ('vocational', 'Vocational Institution', 'peerfora_starter_directory'),
    ('online', 'Online Institution', 'peerfora_starter_directory'),
    ('research', 'Research Institution', 'peerfora_starter_directory'),
    ('professional_training', 'Professional Training', 'peerfora_starter_directory'),
    ('other', 'Other', 'peerfora_starter_directory');

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
