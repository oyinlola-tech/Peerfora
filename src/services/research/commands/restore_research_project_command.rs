use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::{
        research::ResearchProjectStatus,
        research_project_enforcement_reason::ResearchProjectEnforcementReason,
    },
    errors::app_error::AppError,
    repositories::research_administration_repository,
};

pub async fn restore_research_project(
    pool: &PgPool,
    project_id: Uuid,
    administrator_id: Uuid,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = ResearchProjectEnforcementReason::try_from(reason_code)?;
    research_administration_repository::change_research_project_status(
        pool,
        project_id,
        administrator_id,
        ResearchProjectStatus::Suspended,
        ResearchProjectStatus::Active,
        reason.as_database_value(),
        correlation_id,
    )
    .await
}
