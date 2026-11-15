use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_project_repository};

pub async fn leave_research_project(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    research_project_repository::leave_research_project(pool, project_id, account_id).await
}
