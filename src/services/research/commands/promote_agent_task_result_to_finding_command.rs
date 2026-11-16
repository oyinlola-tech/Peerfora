use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::research_task_repository,
    validations::research_finding_validation::ValidatedResearchFinding,
};

pub async fn promote_approved_agent_task_result_to_finding(
    pool: &PgPool,
    task_id: Uuid,
    submission_number: i16,
    promoter_account_id: Uuid,
    finding_id: Uuid,
    finding: &ValidatedResearchFinding,
) -> Result<(), AppError> {
    research_task_repository::promote_approved_agent_submission_to_finding(
        pool,
        task_id,
        submission_number,
        promoter_account_id,
        finding_id,
        finding,
    )
    .await
}
