use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::discussions::{DEFAULT_COMMUNITY_POSTS_LIMIT, MAX_COMMUNITY_POSTS_LIMIT},
    errors::app_error::AppError,
    models::discussion::CommunityPost,
    repositories::discussion_repository,
};

pub async fn list_community_posts(
    pool: &PgPool,
    community_slug: &str,
    viewer_account_id: Option<Uuid>,
    requested_limit: Option<i64>,
) -> Result<Vec<CommunityPost>, AppError> {
    let result_limit = requested_limit
        .unwrap_or(DEFAULT_COMMUNITY_POSTS_LIMIT)
        .clamp(1, MAX_COMMUNITY_POSTS_LIMIT);
    discussion_repository::list_community_posts_visible_to_account(
        pool,
        community_slug,
        viewer_account_id,
        result_limit,
    )
    .await
}
