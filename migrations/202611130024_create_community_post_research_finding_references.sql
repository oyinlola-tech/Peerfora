CREATE TABLE community_post_research_findings (
    post_id UUID NOT NULL REFERENCES community_posts(id) ON DELETE RESTRICT,
    finding_id UUID NOT NULL REFERENCES research_findings(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (post_id, finding_id)
);

CREATE INDEX community_post_research_findings_finding_idx
    ON community_post_research_findings (finding_id, created_at DESC);
