use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_project_repository};

pub async fn archive_research_project(
    pool: &PgPool,
    project_id: Uuid,
    owner_account_id: Uuid,
) -> Result<(), AppError> {
    research_project_repository::archive_research_project(pool, project_id, owner_account_id).await
}
