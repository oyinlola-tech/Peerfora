pub const COMMUNITY_POSTS_ROUTE: &str = "/api/v1/communities/{slug}/posts";
pub const MIN_COMMUNITY_POST_TITLE_LENGTH: usize = 3;
pub const MAX_COMMUNITY_POST_TITLE_LENGTH: usize = 200;
pub const MAX_COMMUNITY_POST_BODY_LENGTH: usize = 20000;
pub const DEFAULT_COMMUNITY_POSTS_LIMIT: i64 = 20;
pub const MAX_COMMUNITY_POSTS_LIMIT: i64 = 50;
pub const POST_COMMENTS_ROUTE: &str = "/api/v1/posts/{post_id}/comments";
pub const POST_RESEARCH_FINDING_REFERENCES_ROUTE: &str =
    "/api/v1/posts/{post_id}/research-findings";
pub const POST_RESEARCH_FINDING_REFERENCE_ROUTE: &str =
    "/api/v1/posts/{post_id}/research-findings/{finding_id}";
pub const MAX_POST_RESEARCH_FINDING_REFERENCES: i64 = 10;
pub const MIN_POST_COMMENT_BODY_LENGTH: usize = 1;
pub const MAX_POST_COMMENT_BODY_LENGTH: usize = 10000;
pub const DEFAULT_POST_COMMENTS_LIMIT: i64 = 50;
pub const MAX_POST_COMMENTS_LIMIT: i64 = 100;
pub const MAX_POST_COMMENTS_OFFSET: i64 = 10000;
