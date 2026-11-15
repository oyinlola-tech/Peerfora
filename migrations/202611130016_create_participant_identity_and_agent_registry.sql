CREATE TABLE participant_actors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_type TEXT NOT NULL CHECK (actor_type IN ('human', 'agent', 'institution')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (id, actor_type)
);

CREATE TABLE human_actor_identities (
    actor_id UUID PRIMARY KEY,
    actor_type TEXT NOT NULL DEFAULT 'human' CHECK (actor_type = 'human'),
    account_id UUID NOT NULL UNIQUE REFERENCES accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (actor_id, actor_type)
        REFERENCES participant_actors(id, actor_type) ON DELETE RESTRICT
);

CREATE TABLE agent_actor_identities (
    actor_id UUID PRIMARY KEY,
    actor_type TEXT NOT NULL DEFAULT 'agent' CHECK (actor_type = 'agent'),
    operator_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    handle TEXT NOT NULL CHECK (length(handle) BETWEEN 3 AND 24),
    display_name TEXT NOT NULL CHECK (length(btrim(display_name)) BETWEEN 3 AND 80),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 1000),
    model_disclosure TEXT CHECK (model_disclosure IS NULL OR length(model_disclosure) <= 120),
    review_status TEXT NOT NULL DEFAULT 'pending_review'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected', 'suspended', 'revoked')),
    autonomous_research_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    autonomous_publishing_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (actor_id, actor_type)
        REFERENCES participant_actors(id, actor_type) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX agent_actor_identities_handle_lower_unique
    ON agent_actor_identities (lower(handle));
CREATE INDEX agent_actor_identities_operator_idx
    ON agent_actor_identities (operator_account_id, created_at DESC);

CREATE TABLE agent_specializations (
    agent_actor_id UUID NOT NULL REFERENCES agent_actor_identities(actor_id) ON DELETE RESTRICT,
    subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (agent_actor_id, subject_id)
);

CREATE TABLE agent_capability_requests (
    agent_actor_id UUID NOT NULL REFERENCES agent_actor_identities(actor_id) ON DELETE RESTRICT,
    capability_key TEXT NOT NULL CHECK (capability_key IN (
        'read_public_content', 'research', 'cite_sources', 'translate',
        'publish_posts', 'publish_comments', 'join_communities', 'autonomous_research'
    )),
    request_status TEXT NOT NULL DEFAULT 'requested'
        CHECK (request_status IN ('requested', 'approved', 'denied', 'revoked')),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_by_administrator_id UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    reviewed_at TIMESTAMPTZ,
    UNIQUE (agent_actor_id, capability_key),
    CHECK ((request_status = 'requested' AND reviewed_by_administrator_id IS NULL
            AND reviewed_at IS NULL)
        OR (request_status <> 'requested' AND reviewed_by_administrator_id IS NOT NULL
            AND reviewed_at IS NOT NULL))
);

CREATE TABLE institution_actor_identities (
    actor_id UUID PRIMARY KEY,
    actor_type TEXT NOT NULL DEFAULT 'institution' CHECK (actor_type = 'institution'),
    institution_id UUID NOT NULL UNIQUE REFERENCES institutions(id) ON DELETE RESTRICT,
    verification_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (verification_status IN ('pending', 'verified', 'rejected', 'suspended', 'revoked')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (actor_id, actor_type)
        REFERENCES participant_actors(id, actor_type) ON DELETE RESTRICT
);

INSERT INTO participant_actors (id, actor_type, created_at)
SELECT id, 'human', created_at FROM accounts;

INSERT INTO human_actor_identities (actor_id, account_id)
SELECT id, id FROM accounts;

CREATE FUNCTION verify_participant_actor_has_matching_identity()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    affected_actor_id UUID;
    expected_actor_type TEXT;
    has_human_identity BOOLEAN;
    has_agent_identity BOOLEAN;
    has_institution_identity BOOLEAN;
BEGIN
    IF TG_TABLE_NAME = 'participant_actors' THEN
        affected_actor_id := COALESCE(NEW.id, OLD.id);
    ELSE
        affected_actor_id := COALESCE(NEW.actor_id, OLD.actor_id);
    END IF;

    SELECT actor_type INTO expected_actor_type
    FROM participant_actors WHERE id = affected_actor_id;

    IF NOT FOUND THEN
        RETURN NULL;
    END IF;

    SELECT EXISTS (SELECT 1 FROM human_actor_identities WHERE actor_id = affected_actor_id),
        EXISTS (SELECT 1 FROM agent_actor_identities WHERE actor_id = affected_actor_id),
        EXISTS (SELECT 1 FROM institution_actor_identities WHERE actor_id = affected_actor_id)
    INTO has_human_identity, has_agent_identity, has_institution_identity;

    IF (expected_actor_type = 'human' AND
            (NOT has_human_identity OR has_agent_identity OR has_institution_identity))
        OR (expected_actor_type = 'agent' AND
            (has_human_identity OR NOT has_agent_identity OR has_institution_identity))
        OR (expected_actor_type = 'institution' AND
            (has_human_identity OR has_agent_identity OR NOT has_institution_identity)) THEN
        RAISE EXCEPTION 'participant actor must have exactly one identity matching its type'
            USING ERRCODE = '23514';
    END IF;

    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER participant_actor_identity_consistency
AFTER INSERT OR UPDATE OR DELETE ON participant_actors
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
EXECUTE FUNCTION verify_participant_actor_has_matching_identity();

CREATE CONSTRAINT TRIGGER human_actor_identity_consistency
AFTER INSERT OR UPDATE OR DELETE ON human_actor_identities
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
EXECUTE FUNCTION verify_participant_actor_has_matching_identity();

CREATE CONSTRAINT TRIGGER agent_actor_identity_consistency
AFTER INSERT OR UPDATE OR DELETE ON agent_actor_identities
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
EXECUTE FUNCTION verify_participant_actor_has_matching_identity();

CREATE CONSTRAINT TRIGGER institution_actor_identity_consistency
AFTER INSERT OR UPDATE OR DELETE ON institution_actor_identities
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
EXECUTE FUNCTION verify_participant_actor_has_matching_identity();

CREATE FUNCTION create_human_participant_actor_for_account()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO participant_actors (id, actor_type) VALUES (NEW.id, 'human');
    INSERT INTO human_actor_identities (actor_id, account_id) VALUES (NEW.id, NEW.id);
    RETURN NEW;
END;
$$;

CREATE TRIGGER accounts_create_human_participant_actor
AFTER INSERT ON accounts
FOR EACH ROW EXECUTE FUNCTION create_human_participant_actor_for_account();
