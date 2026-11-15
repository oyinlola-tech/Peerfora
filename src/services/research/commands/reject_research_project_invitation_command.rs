use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::research_project_repository};

pub async fn reject_research_project_invitation(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    research_project_repository::reject_research_project_invitation(pool, project_id, account_id)
        .await
}
