use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::research_project::ResearchProject,
    repositories::research_project_repository,
};

pub async fn find_research_project_for_account(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Option<Uuid>,
) -> Result<Option<ResearchProject>, AppError> {
    let actor_id = match account_id {
        Some(account_id) => sqlx::query_scalar::<_, Uuid>(
            "SELECT actor_id FROM human_actor_identities WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::Internal)?,
        None => None,
    };
    research_project_repository::find_research_project_visible_to_actor(pool, project_id, actor_id)
        .await
}
