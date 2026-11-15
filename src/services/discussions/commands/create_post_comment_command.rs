use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::discussions::{MAX_POST_COMMENT_BODY_LENGTH, MIN_POST_COMMENT_BODY_LENGTH},
    enums::discussion::DiscussionError,
    errors::app_error::AppError,
    models::discussion::CommunityPostComment,
    repositories::discussion_repository,
};

pub async fn create_post_comment(
    pool: &PgPool,
    post_id: Uuid,
    author_account_id: Uuid,
    parent_comment_id: Option<Uuid>,
    body: &str,
) -> Result<CommunityPostComment, AppError> {
    let body = body.trim();
    if !(MIN_POST_COMMENT_BODY_LENGTH..=MAX_POST_COMMENT_BODY_LENGTH)
        .contains(&body.chars().count())
    {
        return Err(DiscussionError::CommentBodyInvalid.into());
    }

    discussion_repository::create_post_comment(
        pool,
        post_id,
        author_account_id,
        parent_comment_id,
        body,
    )
    .await
}
