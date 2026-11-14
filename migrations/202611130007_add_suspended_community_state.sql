ALTER TABLE communities
    DROP CONSTRAINT communities_lifecycle_status_check,
    ADD CONSTRAINT communities_lifecycle_status_check
        CHECK (lifecycle_status IN ('active', 'suspended', 'archived'));

CREATE INDEX communities_admin_name_prefix_idx
    ON communities (lower(display_name) text_pattern_ops);

CREATE INDEX communities_admin_status_category_created_idx
    ON communities (lifecycle_status, category, created_at DESC, id);
