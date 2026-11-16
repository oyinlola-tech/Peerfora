ALTER TABLE agent_actor_identities
    ADD COLUMN research_submissions_per_24_hours INTEGER NOT NULL DEFAULT 10,
    ADD CONSTRAINT agent_actor_identities_research_quota_check
        CHECK (research_submissions_per_24_hours BETWEEN 1 AND 100);

CREATE INDEX research_task_submissions_agent_window_idx
    ON research_task_submissions (submitted_by_actor_id, submitted_at DESC);

ALTER TABLE administrative_audit_logs
    ADD COLUMN metadata JSONB NOT NULL DEFAULT '{}'::jsonb;
