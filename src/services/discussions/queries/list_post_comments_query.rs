use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::discussions::{
        DEFAULT_POST_COMMENTS_LIMIT, MAX_POST_COMMENTS_LIMIT, MAX_POST_COMMENTS_OFFSET,
    },
    errors::app_error::AppError,
    models::discussion::CommunityPostComment,
    repositories::discussion_repository,
};

pub async fn list_post_comments(
    pool: &PgPool,
    post_id: Uuid,
    viewer_account_id: Option<Uuid>,
    requested_limit: Option<i64>,
    requested_offset: Option<i64>,
) -> Result<Vec<CommunityPostComment>, AppError> {
    let result_limit = requested_limit
        .unwrap_or(DEFAULT_POST_COMMENTS_LIMIT)
        .clamp(1, MAX_POST_COMMENTS_LIMIT);
    let result_offset = requested_offset
        .unwrap_or_default()
        .clamp(0, MAX_POST_COMMENTS_OFFSET);
    discussion_repository::list_post_comments_visible_to_account(
        pool,
        post_id,
        viewer_account_id,
        result_limit,
        result_offset,
    )
    .await
}
