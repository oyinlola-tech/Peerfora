use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::research::{DEFAULT_RESEARCH_PROJECT_LIST_LIMIT, MAX_RESEARCH_PROJECT_LIST_LIMIT},
    errors::app_error::AppError,
    models::research_project::ResearchProject,
    repositories::research_project_repository,
};

pub async fn list_research_projects_for_account(
    pool: &PgPool,
    account_id: Uuid,
    requested_limit: Option<i64>,
) -> Result<Vec<ResearchProject>, AppError> {
    let result_limit = requested_limit
        .unwrap_or(DEFAULT_RESEARCH_PROJECT_LIST_LIMIT)
        .clamp(1, MAX_RESEARCH_PROJECT_LIST_LIMIT);
    let actor_id: Uuid =
        sqlx::query_scalar("SELECT actor_id FROM human_actor_identities WHERE account_id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| AppError::Internal)?
            .ok_or(crate::enums::research::ResearchProjectError::InviteeUnavailable)?;
    research_project_repository::list_research_projects_visible_to_actor(
        pool,
        actor_id,
        result_limit,
    )
    .await
}
