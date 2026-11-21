use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::communication_repository::notification_repository,
};

pub async fn update_notification_preferences(
    pool: &PgPool,
    account_id: Uuid,
    in_app_enabled: bool,
    email_enabled: bool,
    push_enabled: bool,
    research_enabled: bool,
    agent_enabled: bool,
    community_enabled: bool,
) -> Result<(), AppError> {
    notification_repository::update_preferences(
        pool,
        account_id,
        in_app_enabled,
        email_enabled,
        push_enabled,
        research_enabled,
        agent_enabled,
        community_enabled,
    )
    .await
}
