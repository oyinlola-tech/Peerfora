CREATE TABLE research_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_actor_id UUID NOT NULL REFERENCES human_actor_identities(actor_id) ON DELETE RESTRICT,
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 3 AND 160),
    abstract TEXT CHECK (abstract IS NULL OR length(abstract) <= 2000),
    description TEXT NOT NULL DEFAULT '' CHECK (length(description) <= 10000),
    objectives TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    research_questions TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    visibility TEXT NOT NULL DEFAULT 'private'
        CHECK (visibility IN ('private', 'invitation_only', 'public')),
    project_status TEXT NOT NULL DEFAULT 'active'
        CHECK (project_status IN ('active', 'archived', 'suspended')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    archived_at TIMESTAMPTZ,
    CHECK (cardinality(objectives) <= 20),
    CHECK (cardinality(research_questions) <= 20),
    CHECK ((project_status = 'archived') = (archived_at IS NOT NULL))
);

CREATE INDEX research_projects_owner_created_idx
    ON research_projects (owner_actor_id, created_at DESC);
CREATE INDEX research_projects_public_created_idx
    ON research_projects (created_at DESC) WHERE visibility = 'public' AND project_status = 'active';

CREATE TABLE research_project_subjects (
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE CASCADE,
    subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, subject_id)
);

CREATE TABLE research_project_members (
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE RESTRICT,
    actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    member_role TEXT NOT NULL CHECK (member_role IN (
        'owner', 'supervisor', 'researcher', 'contributor', 'reviewer', 'observer'
    )),
    membership_status TEXT NOT NULL CHECK (membership_status IN (
        'invited', 'active', 'rejected', 'removed', 'left'
    )),
    invited_by_actor_id UUID REFERENCES participant_actors(id) ON DELETE RESTRICT,
    invited_at TIMESTAMPTZ,
    joined_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, actor_id),
    CHECK (membership_status <> 'invited' OR invited_at IS NOT NULL),
    CHECK (membership_status <> 'active' OR joined_at IS NOT NULL),
    CHECK (membership_status <> 'invited' OR invited_by_actor_id IS NOT NULL)
);

CREATE INDEX research_project_members_actor_status_idx
    ON research_project_members (actor_id, membership_status, updated_at DESC);
CREATE INDEX research_project_members_project_status_idx
    ON research_project_members (project_id, membership_status, member_role);

CREATE FUNCTION enforce_research_project_owner_membership()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    affected_project_id UUID;
    owner_actor UUID;
BEGIN
    IF TG_TABLE_NAME = 'research_projects' THEN
        affected_project_id := COALESCE(NEW.id, OLD.id);
    ELSE
        affected_project_id := COALESCE(NEW.project_id, OLD.project_id);
    END IF;

    SELECT owner_actor_id INTO owner_actor
    FROM research_projects WHERE id = affected_project_id;
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM research_project_members
        WHERE project_id = affected_project_id AND actor_id = owner_actor
            AND member_role = 'owner' AND membership_status = 'active'
    ) OR EXISTS (
        SELECT 1 FROM research_project_members
        WHERE project_id = affected_project_id AND member_role = 'owner'
            AND membership_status = 'active' AND actor_id <> owner_actor
    ) THEN
        RAISE EXCEPTION 'research project owner and active owner membership must match'
            USING ERRCODE = '23514';
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER research_projects_owner_membership_consistency
AFTER INSERT OR UPDATE OR DELETE ON research_projects
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
EXECUTE FUNCTION enforce_research_project_owner_membership();

CREATE CONSTRAINT TRIGGER research_project_members_owner_consistency
AFTER INSERT OR UPDATE OR DELETE ON research_project_members
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
EXECUTE FUNCTION enforce_research_project_owner_membership();
