use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_project_repository};

pub async fn remove_research_project_member(
    pool: &PgPool,
    project_id: Uuid,
    owner_account_id: Uuid,
    username: &str,
) -> Result<(), AppError> {
    research_project_repository::remove_research_project_member(
        pool,
        project_id,
        owner_account_id,
        username.trim(),
    )
    .await
}
