use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::discussions::{
        MAX_COMMUNITY_POST_BODY_LENGTH, MAX_COMMUNITY_POST_TITLE_LENGTH,
        MIN_COMMUNITY_POST_TITLE_LENGTH,
    },
    enums::discussion::{CommunityPostKind, DiscussionError},
    errors::app_error::AppError,
    models::discussion::CommunityPost,
    repositories::discussion_repository,
};

pub async fn create_community_post(
    pool: &PgPool,
    community_slug: &str,
    author_account_id: Uuid,
    kind: CommunityPostKind,
    title: &str,
    body: &str,
) -> Result<CommunityPost, AppError> {
    let title = title.trim();
    let body = body.trim();
    if !(MIN_COMMUNITY_POST_TITLE_LENGTH..=MAX_COMMUNITY_POST_TITLE_LENGTH)
        .contains(&title.chars().count())
    {
        return Err(DiscussionError::TitleInvalid.into());
    }
    if !(1..=MAX_COMMUNITY_POST_BODY_LENGTH).contains(&body.chars().count()) {
        return Err(DiscussionError::BodyInvalid.into());
    }

    discussion_repository::create_community_post(
        pool,
        community_slug,
        author_account_id,
        kind.as_database_value(),
        title,
        body,
    )
    .await
}
