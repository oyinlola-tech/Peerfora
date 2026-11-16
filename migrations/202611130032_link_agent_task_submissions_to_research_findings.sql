ALTER TABLE research_task_submissions
    ADD COLUMN promoted_finding_id UUID UNIQUE
        REFERENCES research_findings(id) ON DELETE RESTRICT;

ALTER TABLE research_task_submissions
    ADD CONSTRAINT research_task_submission_promotion_requires_approval
    CHECK (promoted_finding_id IS NULL OR review_status = 'approved');
