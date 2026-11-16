CREATE TABLE knowledge_citations (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 300),
    doi TEXT CHECK (doi IS NULL OR length(btrim(doi)) BETWEEN 1 AND 255),
    external_url TEXT CHECK (external_url IS NULL OR length(btrim(external_url)) <= 2048),
    authors JSONB NOT NULL DEFAULT '[]'::jsonb CHECK (jsonb_typeof(authors) = 'array'),
    publication_year SMALLINT CHECK (publication_year IS NULL OR publication_year BETWEEN 1000 AND 2200),
    license TEXT CHECK (license IS NULL OR length(btrim(license)) <= 200),
    provenance TEXT NOT NULL DEFAULT 'user_submitted',
    created_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (doi IS NOT NULL OR external_url IS NOT NULL)
);

CREATE TABLE knowledge_entries (
    id UUID PRIMARY KEY,
    academic_subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE RESTRICT,
    created_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    entry_status TEXT NOT NULL CHECK (entry_status IN (
        'submitted', 'approved', 'published', 'rejected', 'corrected', 'retracted'
    )),
    latest_revision_number SMALLINT NOT NULL CHECK (latest_revision_number > 0),
    published_revision_number SMALLINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (id, academic_subject_id),
    CHECK ((entry_status IN ('published', 'corrected', 'retracted')) =
        (published_revision_number IS NOT NULL))
);

CREATE INDEX knowledge_entries_subject_status_idx
    ON knowledge_entries (academic_subject_id, entry_status, updated_at DESC);
CREATE INDEX knowledge_entries_publication_idx
    ON knowledge_entries (updated_at DESC) WHERE entry_status IN ('published', 'corrected');

CREATE TABLE knowledge_entry_revisions (
    entry_id UUID NOT NULL REFERENCES knowledge_entries(id) ON DELETE RESTRICT,
    revision_number SMALLINT NOT NULL CHECK (revision_number > 0),
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 3 AND 180),
    summary TEXT NOT NULL CHECK (length(btrim(summary)) BETWEEN 20 AND 3000),
    search_document TSVECTOR GENERATED ALWAYS AS
        (to_tsvector('simple', title || ' ' || summary)) STORED,
    revision_status TEXT NOT NULL CHECK (revision_status IN (
        'submitted', 'approved', 'published', 'rejected', 'superseded'
    )),
    authored_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    reviewed_by_administrator_id UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    review_reason TEXT CHECK (review_reason IS NULL OR length(btrim(review_reason)) <= 1000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    PRIMARY KEY (entry_id, revision_number),
    CHECK (
        (
            revision_status IN ('approved', 'published', 'rejected', 'superseded')
            AND reviewed_by_administrator_id IS NOT NULL
            AND reviewed_at IS NOT NULL
        ) OR (
            revision_status = 'submitted'
            AND reviewed_by_administrator_id IS NULL
            AND reviewed_at IS NULL
        )
    )
);

CREATE INDEX knowledge_entry_revisions_search_idx
    ON knowledge_entry_revisions USING GIN (search_document);

ALTER TABLE knowledge_entries
    ADD CONSTRAINT knowledge_entries_latest_revision_fk
    FOREIGN KEY (id, latest_revision_number)
    REFERENCES knowledge_entry_revisions (entry_id, revision_number)
    DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE knowledge_entries
    ADD CONSTRAINT knowledge_entries_published_revision_fk
    FOREIGN KEY (id, published_revision_number)
    REFERENCES knowledge_entry_revisions (entry_id, revision_number)
    DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE knowledge_revision_citations (
    entry_id UUID NOT NULL,
    revision_number SMALLINT NOT NULL,
    citation_id UUID NOT NULL REFERENCES knowledge_citations(id) ON DELETE RESTRICT,
    PRIMARY KEY (entry_id, revision_number, citation_id),
    FOREIGN KEY (entry_id, revision_number)
        REFERENCES knowledge_entry_revisions (entry_id, revision_number) ON DELETE RESTRICT
);

CREATE TABLE knowledge_revision_research_findings (
    entry_id UUID NOT NULL,
    revision_number SMALLINT NOT NULL,
    finding_id UUID NOT NULL REFERENCES research_findings(id) ON DELETE RESTRICT,
    relation TEXT NOT NULL CHECK (relation IN ('supports', 'contradicts', 'contextualizes')),
    PRIMARY KEY (entry_id, revision_number, finding_id),
    FOREIGN KEY (entry_id, revision_number)
        REFERENCES knowledge_entry_revisions (entry_id, revision_number) ON DELETE RESTRICT
);

INSERT INTO permissions (permission_key, description, is_assignable, is_sensitive)
VALUES
    ('knowledge.entries.read', 'Read knowledge entries in administration', TRUE, FALSE),
    ('knowledge.proposals.review', 'Review submitted knowledge proposals', TRUE, TRUE),
    ('knowledge.entries.publish', 'Publish approved knowledge entries', TRUE, TRUE)
ON CONFLICT (permission_key) DO UPDATE
SET description = EXCLUDED.description,
    is_assignable = EXCLUDED.is_assignable,
    is_sensitive = EXCLUDED.is_sensitive;

INSERT INTO roles (role_key, display_name, description, is_system_role, is_active)
VALUES (
    'knowledge_editor',
    'Knowledge Editor',
    'Review and publish sourced knowledge proposals',
    TRUE,
    TRUE
)
ON CONFLICT (role_key) DO NOTHING;

INSERT INTO role_permissions (role_key, permission_key)
VALUES
    ('knowledge_editor', 'knowledge.entries.read'),
    ('knowledge_editor', 'knowledge.proposals.review'),
    ('knowledge_editor', 'knowledge.entries.publish'),
    ('super_administrator', 'knowledge.entries.read'),
    ('super_administrator', 'knowledge.proposals.review'),
    ('super_administrator', 'knowledge.entries.publish')
ON CONFLICT DO NOTHING;
