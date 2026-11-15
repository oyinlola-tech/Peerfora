use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::research_finding_repository,
    validations::research_finding_validation::ValidatedResearchFinding,
};

pub async fn update_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
    details: ValidatedResearchFinding,
) -> Result<(), AppError> {
    research_finding_repository::update_research_finding(pool, finding_id, account_id, details)
        .await
}
