use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::community_repository};

pub async fn leave_community(
    pool: &PgPool,
    community_slug: &str,
    account_id: Uuid,
) -> Result<(), AppError> {
    community_repository::leave_community(pool, community_slug, account_id).await
}
