use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::discussion::DiscussionError, errors::app_error::AppError,
    repositories::community_post_research_finding_repository,
    services::research::queries::find_public_research_finding_query,
};

pub async fn link_public_research_finding_to_post(
    pool: &PgPool,
    post_id: Uuid,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let public_finding =
        find_public_research_finding_query::find_public_research_finding(pool, finding_id).await?;
    if public_finding.is_none() {
        return Err(DiscussionError::ResearchFindingUnavailable.into());
    }
    community_post_research_finding_repository::create_public_research_finding_reference(
        pool, post_id, finding_id, account_id,
    )
    .await
}
