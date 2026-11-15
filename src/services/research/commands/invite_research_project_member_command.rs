use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::account_auth::{MAX_USERNAME_LENGTH, MIN_USERNAME_LENGTH},
    enums::{account_auth_error::AccountAuthError, research::ResearchProjectMemberRole},
    errors::app_error::AppError,
    repositories::research_project_repository,
};

pub async fn invite_research_project_member(
    pool: &PgPool,
    project_id: Uuid,
    inviter_account_id: Uuid,
    username: &str,
    role: ResearchProjectMemberRole,
) -> Result<(), AppError> {
    let normalized_username = username.trim();
    if !(MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&normalized_username.len())
        || !normalized_username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(AccountAuthError::UsernameInvalid.into());
    }
    research_project_repository::invite_research_project_member(
        pool,
        project_id,
        inviter_account_id,
        normalized_username,
        role,
    )
    .await
}
