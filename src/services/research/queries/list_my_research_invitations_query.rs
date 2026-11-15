use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::research_project::ResearchInvitation,
    repositories::research_project_repository,
};

pub async fn list_my_research_project_invitations(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<ResearchInvitation>, AppError> {
    research_project_repository::list_my_research_invitations(pool, account_id).await
}
