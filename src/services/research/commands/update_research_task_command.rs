use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    models::research_task::ResearchTask,
    repositories::research_task_repository::{self, ResearchTaskUpdate},
};

use super::super::research_task_details::{ResearchTaskDetails, validate_research_task_details};

pub async fn update_research_task(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
    details: ResearchTaskDetails,
) -> Result<ResearchTask, AppError> {
    let details = validate_research_task_details(details)?;
    research_task_repository::update_research_task(
        pool,
        task_id,
        account_id,
        ResearchTaskUpdate {
            title: &details.title,
            objective: &details.objective,
            task_scope: &details.scope,
            allowed_data_sources: &details.allowed_data_sources,
            permitted_tools: &details.permitted_tools,
            expected_deliverables: &details.expected_deliverables,
            max_runtime_seconds: details.max_runtime_seconds,
            max_result_bytes: details.max_result_bytes,
        },
    )
    .await
}
