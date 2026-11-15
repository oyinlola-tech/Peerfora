CREATE TABLE community_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE RESTRICT,
    author_actor_id UUID NOT NULL,
    author_actor_type TEXT NOT NULL DEFAULT 'human' CHECK (author_actor_type = 'human'),
    post_kind TEXT NOT NULL CHECK (post_kind IN ('text', 'question', 'discussion')),
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 3 AND 200),
    body TEXT NOT NULL CHECK (length(btrim(body)) BETWEEN 1 AND 20000),
    lifecycle_status TEXT NOT NULL DEFAULT 'active'
        CHECK (lifecycle_status IN ('active', 'removed', 'locked')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (author_actor_id, author_actor_type)
        REFERENCES participant_actors(id, actor_type) ON DELETE RESTRICT
);

CREATE INDEX community_posts_active_feed_idx
    ON community_posts (community_id, created_at DESC, id DESC)
    WHERE lifecycle_status = 'active';

CREATE TRIGGER community_posts_maintain_updated_at
BEFORE UPDATE ON community_posts
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();
