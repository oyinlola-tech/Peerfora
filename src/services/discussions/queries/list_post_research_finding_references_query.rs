use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::research_finding::ResearchFinding,
    repositories::community_post_research_finding_repository,
    services::research::queries::find_public_research_finding_query,
};

pub async fn list_post_research_finding_references(
    pool: &PgPool,
    post_id: Uuid,
    viewer_account_id: Option<Uuid>,
) -> Result<Vec<ResearchFinding>, AppError> {
    let finding_ids =
        community_post_research_finding_repository::list_research_finding_reference_ids_for_visible_post(
            pool,
            post_id,
            viewer_account_id,
        )
        .await?;
    let mut findings = Vec::with_capacity(finding_ids.len());
    for finding_id in finding_ids {
        if let Some(finding) =
            find_public_research_finding_query::find_public_research_finding(pool, finding_id)
                .await?
        {
            findings.push(finding);
        }
    }
    Ok(findings)
}
