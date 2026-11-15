use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    models::admin_research::AdministrativeResearchProjectRecord,
    repositories::research_administration_repository,
};

pub async fn find_research_project_for_administration(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<AdministrativeResearchProjectRecord, AppError> {
    research_administration_repository::find_research_project_for_administration(
        pool,
        project_id,
        administrator_id,
        correlation_id,
    )
    .await
}
