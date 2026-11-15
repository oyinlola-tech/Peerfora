use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::research_finding::ResearchSource,
    repositories::research_finding_repository,
    validations::research_finding_validation::ValidatedResearchSource,
};

pub async fn create_research_source(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    source_id: Uuid,
    source: ValidatedResearchSource,
) -> Result<ResearchSource, AppError> {
    research_finding_repository::create_research_source(
        pool, project_id, account_id, source_id, source,
    )
    .await
}
