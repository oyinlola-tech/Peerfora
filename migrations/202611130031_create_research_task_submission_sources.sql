CREATE TABLE research_task_submission_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES research_task_submissions(id) ON DELETE RESTRICT,
    source_order SMALLINT NOT NULL CHECK (source_order BETWEEN 1 AND 50),
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 500),
    source_url TEXT NOT NULL CHECK (length(source_url) BETWEEN 1 AND 2048),
    doi TEXT CHECK (doi IS NULL OR length(doi) BETWEEN 1 AND 255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (submission_id, source_order),
    UNIQUE (submission_id, source_url)
);

CREATE INDEX research_task_submission_sources_submission_idx
    ON research_task_submission_sources (submission_id, source_order);
