use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::community::Community, repositories::community_repository,
};

pub async fn find_community_visible_to_account(
    pool: &PgPool,
    community_slug: &str,
    account_id: Option<Uuid>,
) -> Result<Option<Community>, AppError> {
    community_repository::find_community_visible_to_account(pool, community_slug, account_id).await
}

pub async fn list_discoverable_communities(
    pool: &PgPool,
    requested_limit: Option<i64>,
) -> Result<Vec<Community>, AppError> {
    let result_limit = requested_limit.unwrap_or(20).clamp(1, 50);
    community_repository::list_discoverable_communities(pool, result_limit).await
}
