use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::community_post_research_finding_repository,
};

pub async fn remove_post_research_finding_reference(
    pool: &PgPool,
    post_id: Uuid,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    community_post_research_finding_repository::remove_post_research_finding_reference(
        pool, post_id, finding_id, account_id,
    )
    .await
}
