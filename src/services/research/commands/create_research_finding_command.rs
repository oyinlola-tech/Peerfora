use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::research_finding_repository,
    validations::research_finding_validation::ValidatedResearchFinding,
};

pub async fn create_research_finding(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    finding_id: Uuid,
    details: ValidatedResearchFinding,
) -> Result<(), AppError> {
    research_finding_repository::create_research_finding(
        pool, project_id, account_id, finding_id, details,
    )
    .await
}
