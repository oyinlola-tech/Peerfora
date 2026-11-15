CREATE TABLE community_post_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES community_posts(id) ON DELETE RESTRICT,
    parent_comment_id UUID,
    author_actor_id UUID NOT NULL,
    author_actor_type TEXT NOT NULL DEFAULT 'human' CHECK (author_actor_type = 'human'),
    body TEXT NOT NULL CHECK (length(btrim(body)) BETWEEN 1 AND 10000),
    lifecycle_status TEXT NOT NULL DEFAULT 'active'
        CHECK (lifecycle_status IN ('active', 'removed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (post_id, id),
    FOREIGN KEY (post_id, parent_comment_id)
        REFERENCES community_post_comments(post_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (author_actor_id, author_actor_type)
        REFERENCES participant_actors(id, actor_type) ON DELETE RESTRICT
);

CREATE INDEX community_post_comments_thread_idx
    ON community_post_comments (post_id, created_at DESC, id DESC)
    WHERE lifecycle_status = 'active';

CREATE TRIGGER community_post_comments_maintain_updated_at
BEFORE UPDATE ON community_post_comments
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
