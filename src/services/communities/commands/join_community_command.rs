use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::community::CommunityMembership,
    repositories::community_repository,
};

pub async fn join_community(
    pool: &PgPool,
    community_slug: &str,
    account_id: Uuid,
) -> Result<CommunityMembership, AppError> {
    community_repository::join_community(pool, community_slug, account_id).await
}
