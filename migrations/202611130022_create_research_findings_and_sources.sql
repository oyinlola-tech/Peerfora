CREATE TABLE research_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE RESTRICT,
    source_type TEXT NOT NULL CHECK (source_type IN ('publication', 'doi', 'external_url', 'dataset')),
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 500),
    doi TEXT CHECK (doi IS NULL OR length(btrim(doi)) BETWEEN 1 AND 255),
    source_url TEXT CHECK (source_url IS NULL OR length(source_url) <= 2048),
    authors TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    publication_year SMALLINT CHECK (publication_year BETWEEN 1000 AND 9999),
    provenance TEXT NOT NULL CHECK (length(btrim(provenance)) BETWEEN 1 AND 2000),
    license TEXT CHECK (license IS NULL OR length(btrim(license)) <= 500),
    created_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (id, project_id),
    CHECK (cardinality(authors) <= 50),
    CHECK (source_type <> 'doi' OR doi IS NOT NULL),
    CHECK (source_type <> 'external_url' OR source_url IS NOT NULL)
);

CREATE UNIQUE INDEX research_sources_project_doi_unique
    ON research_sources (project_id, lower(doi)) WHERE doi IS NOT NULL;
CREATE INDEX research_sources_project_created_idx
    ON research_sources (project_id, created_at DESC);

CREATE TABLE research_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE RESTRICT,
    created_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    finding_status TEXT NOT NULL CHECK (finding_status IN (
        'draft', 'in_review', 'approved', 'published', 'correction_draft',
        'correction_in_review', 'correction_approved', 'retracted'
    )),
    latest_revision_number SMALLINT NOT NULL DEFAULT 1 CHECK (latest_revision_number BETWEEN 1 AND 100),
    published_revision_number SMALLINT CHECK (published_revision_number BETWEEN 1 AND 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    retracted_at TIMESTAMPTZ,
    UNIQUE (id, project_id),
    CHECK ((finding_status = 'retracted') = (retracted_at IS NOT NULL)),
    CHECK (finding_status <> 'retracted' OR published_revision_number IS NOT NULL),
    CHECK (finding_status <> 'published' OR published_revision_number IS NOT NULL),
    CHECK (finding_status NOT IN ('draft', 'in_review', 'approved') OR published_revision_number IS NULL)
);

CREATE INDEX research_findings_project_status_updated_idx
    ON research_findings (project_id, finding_status, updated_at DESC);
CREATE INDEX research_findings_publication_idx
    ON research_findings (updated_at DESC) WHERE finding_status = 'published';

CREATE TABLE research_finding_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    finding_id UUID NOT NULL,
    project_id UUID NOT NULL,
    revision_number SMALLINT NOT NULL CHECK (revision_number BETWEEN 1 AND 100),
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 3 AND 200),
    statement TEXT NOT NULL CHECK (length(btrim(statement)) BETWEEN 1 AND 10000),
    details TEXT NOT NULL DEFAULT '' CHECK (length(details) <= 20000),
    authored_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    review_status TEXT NOT NULL CHECK (review_status IN (
        'draft', 'in_review', 'approved', 'rejected', 'published', 'superseded'
    )),
    reviewed_by_actor_id UUID REFERENCES participant_actors(id) ON DELETE RESTRICT,
    reviewer_notes TEXT CHECK (reviewer_notes IS NULL OR length(reviewer_notes) <= 5000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    UNIQUE (finding_id, revision_number),
    UNIQUE (finding_id, project_id, revision_number),
    FOREIGN KEY (finding_id, project_id)
        REFERENCES research_findings(id, project_id) ON DELETE RESTRICT,
    CHECK (
        (review_status IN ('draft', 'in_review') AND reviewed_by_actor_id IS NULL AND reviewed_at IS NULL)
        OR (review_status IN ('approved', 'rejected', 'published', 'superseded')
            AND reviewed_by_actor_id IS NOT NULL AND reviewed_at IS NOT NULL)
    ),
    CHECK (reviewed_by_actor_id IS NULL OR reviewed_by_actor_id <> authored_by_actor_id)
);

ALTER TABLE research_findings
    ADD CONSTRAINT research_findings_latest_revision_fk
    FOREIGN KEY (id, project_id, latest_revision_number)
    REFERENCES research_finding_revisions (finding_id, project_id, revision_number)
    DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE research_findings
    ADD CONSTRAINT research_findings_published_revision_fk
    FOREIGN KEY (id, project_id, published_revision_number)
    REFERENCES research_finding_revisions (finding_id, project_id, revision_number)
    DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX research_finding_revisions_finding_created_idx
    ON research_finding_revisions (finding_id, revision_number DESC);

CREATE TABLE research_finding_revision_sources (
    finding_id UUID NOT NULL,
    project_id UUID NOT NULL,
    revision_number SMALLINT NOT NULL,
    source_id UUID NOT NULL,
    citation_locator TEXT CHECK (citation_locator IS NULL OR length(citation_locator) <= 500),
    citation_context TEXT CHECK (citation_context IS NULL OR length(citation_context) <= 2000),
    added_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (finding_id, revision_number, source_id),
    FOREIGN KEY (finding_id, project_id, revision_number)
        REFERENCES research_finding_revisions (finding_id, project_id, revision_number) ON DELETE RESTRICT,
    FOREIGN KEY (source_id, project_id)
        REFERENCES research_sources (id, project_id) ON DELETE RESTRICT
);

CREATE INDEX research_finding_revision_sources_source_idx
    ON research_finding_revision_sources (source_id, created_at DESC);
