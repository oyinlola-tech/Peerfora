use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::account_auth::{MAX_USERNAME_LENGTH, MIN_USERNAME_LENGTH},
    enums::research::ResearchTaskError,
    errors::app_error::AppError,
    repositories::research_task_repository,
};

pub async fn assign_research_task(
    pool: &PgPool,
    task_id: Uuid,
    manager_account_id: Uuid,
    assignee_username: &str,
) -> Result<(), AppError> {
    let username = assignee_username.trim();
    if !(MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&username.len())
        || !username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(ResearchTaskError::AssigneeUsernameInvalid.into());
    }
    research_task_repository::assign_research_task(pool, task_id, manager_account_id, username)
        .await
}
