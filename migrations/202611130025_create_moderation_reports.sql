CREATE TABLE moderation_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    target_account_id UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    target_community_id UUID REFERENCES communities(id) ON DELETE RESTRICT,
    target_post_id UUID REFERENCES community_posts(id) ON DELETE RESTRICT,
    target_comment_id UUID REFERENCES community_post_comments(id) ON DELETE RESTRICT,
    target_research_project_id UUID REFERENCES research_projects(id) ON DELETE RESTRICT,
    target_research_finding_id UUID REFERENCES research_findings(id) ON DELETE RESTRICT,
    category TEXT NOT NULL CHECK (category IN (
        'spam', 'harassment', 'bullying', 'hate_and_discrimination', 'threats',
        'doxxing', 'sexual_exploitation', 'impersonation', 'scams', 'malicious_links',
        'academic_cheating', 'copyright', 'illegal_content', 'coordinated_abuse', 'other'
    )),
    description TEXT NOT NULL CHECK (length(btrim(description)) BETWEEN 1 AND 2000),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'assigned', 'resolved', 'dismissed')),
    assigned_administrator_id UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    resolved_by_administrator_id UUID REFERENCES administrators(id) ON DELETE RESTRICT,
    resolution_reason TEXT CHECK (resolution_reason IS NULL OR length(btrim(resolution_reason)) BETWEEN 1 AND 1000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    CHECK (num_nonnulls(
        target_account_id, target_community_id, target_post_id, target_comment_id,
        target_research_project_id, target_research_finding_id
    ) = 1),
    CHECK (target_account_id IS NULL OR target_account_id <> reporter_account_id),
    CHECK ((status IN ('resolved', 'dismissed')) = (resolved_at IS NOT NULL)),
    CHECK ((status IN ('resolved', 'dismissed')) = (resolved_by_administrator_id IS NOT NULL)),
    CHECK ((status IN ('resolved', 'dismissed')) = (resolution_reason IS NOT NULL))
);

CREATE INDEX moderation_reports_queue_idx ON moderation_reports (status, created_at, id);
CREATE INDEX moderation_reports_reporter_recent_idx ON moderation_reports (reporter_account_id, created_at DESC);
CREATE INDEX moderation_reports_account_target_idx ON moderation_reports (target_account_id, created_at DESC) WHERE target_account_id IS NOT NULL;
CREATE INDEX moderation_reports_community_target_idx ON moderation_reports (target_community_id, created_at DESC) WHERE target_community_id IS NOT NULL;
CREATE INDEX moderation_reports_post_target_idx ON moderation_reports (target_post_id, created_at DESC) WHERE target_post_id IS NOT NULL;
CREATE INDEX moderation_reports_comment_target_idx ON moderation_reports (target_comment_id, created_at DESC) WHERE target_comment_id IS NOT NULL;
CREATE INDEX moderation_reports_project_target_idx ON moderation_reports (target_research_project_id, created_at DESC) WHERE target_research_project_id IS NOT NULL;
CREATE INDEX moderation_reports_finding_target_idx ON moderation_reports (target_research_finding_id, created_at DESC) WHERE target_research_finding_id IS NOT NULL;

CREATE UNIQUE INDEX moderation_reports_active_account_report_unique
    ON moderation_reports (reporter_account_id, target_account_id, category)
    WHERE target_account_id IS NOT NULL AND status IN ('open', 'assigned');
CREATE UNIQUE INDEX moderation_reports_active_community_report_unique
    ON moderation_reports (reporter_account_id, target_community_id, category)
    WHERE target_community_id IS NOT NULL AND status IN ('open', 'assigned');
CREATE UNIQUE INDEX moderation_reports_active_post_report_unique
    ON moderation_reports (reporter_account_id, target_post_id, category)
    WHERE target_post_id IS NOT NULL AND status IN ('open', 'assigned');
CREATE UNIQUE INDEX moderation_reports_active_comment_report_unique
    ON moderation_reports (reporter_account_id, target_comment_id, category)
    WHERE target_comment_id IS NOT NULL AND status IN ('open', 'assigned');
CREATE UNIQUE INDEX moderation_reports_active_project_report_unique
    ON moderation_reports (reporter_account_id, target_research_project_id, category)
    WHERE target_research_project_id IS NOT NULL AND status IN ('open', 'assigned');
CREATE UNIQUE INDEX moderation_reports_active_finding_report_unique
    ON moderation_reports (reporter_account_id, target_research_finding_id, category)
    WHERE target_research_finding_id IS NOT NULL AND status IN ('open', 'assigned');

CREATE TRIGGER moderation_reports_maintain_updated_at
BEFORE UPDATE ON moderation_reports
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
