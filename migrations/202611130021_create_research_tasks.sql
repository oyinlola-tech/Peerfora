CREATE TABLE research_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE RESTRICT,
    created_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 3 AND 160),
    objective TEXT NOT NULL CHECK (length(btrim(objective)) BETWEEN 1 AND 3000),
    task_scope TEXT NOT NULL CHECK (length(btrim(task_scope)) BETWEEN 1 AND 5000),
    allowed_data_sources TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    permitted_tools TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    expected_deliverables TEXT[] NOT NULL,
    max_runtime_seconds INTEGER NOT NULL DEFAULT 1800
        CHECK (max_runtime_seconds BETWEEN 1 AND 86400),
    max_result_bytes INTEGER NOT NULL DEFAULT 200000
        CHECK (max_result_bytes BETWEEN 1 AND 1000000),
    task_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (task_status IN (
            'draft', 'queued', 'assigned', 'running', 'awaiting_review',
            'completed', 'failed', 'cancelled'
        )),
    assigned_actor_id UUID REFERENCES participant_actors(id) ON DELETE RESTRICT,
    assignment_accepted_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (cardinality(allowed_data_sources) <= 20),
    CHECK (cardinality(permitted_tools) <= 20),
    CHECK (cardinality(expected_deliverables) BETWEEN 1 AND 20),
    CHECK (task_status NOT IN (
        'assigned', 'running', 'awaiting_review', 'completed', 'failed'
    ) OR assigned_actor_id IS NOT NULL),
    CHECK (assignment_accepted_at IS NULL OR assigned_actor_id IS NOT NULL),
    CHECK ((task_status = 'cancelled') = (cancelled_at IS NOT NULL))
);

CREATE INDEX research_tasks_project_status_updated_idx
    ON research_tasks (project_id, task_status, updated_at DESC);
CREATE INDEX research_tasks_assignee_status_updated_idx
    ON research_tasks (assigned_actor_id, task_status, updated_at DESC)
    WHERE assigned_actor_id IS NOT NULL;

CREATE TABLE research_task_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES research_tasks(id) ON DELETE RESTRICT,
    submission_number SMALLINT NOT NULL CHECK (submission_number BETWEEN 1 AND 100),
    submitted_by_actor_id UUID NOT NULL REFERENCES participant_actors(id) ON DELETE RESTRICT,
    result TEXT NOT NULL CHECK (octet_length(result) BETWEEN 1 AND 1000000),
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN ('pending', 'approved', 'revision_requested', 'rejected')),
    reviewed_by_actor_id UUID REFERENCES participant_actors(id) ON DELETE RESTRICT,
    reviewer_notes TEXT CHECK (reviewer_notes IS NULL OR octet_length(reviewer_notes) <= 5000),
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    UNIQUE (task_id, submission_number),
    CHECK (
        (review_status = 'pending' AND reviewed_by_actor_id IS NULL AND reviewed_at IS NULL)
        OR
        (review_status <> 'pending' AND reviewed_by_actor_id IS NOT NULL AND reviewed_at IS NOT NULL
            AND reviewed_by_actor_id <> submitted_by_actor_id)
    )
);

CREATE INDEX research_task_submissions_task_number_idx
    ON research_task_submissions (task_id, submission_number DESC);
