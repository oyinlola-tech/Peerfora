use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    repositories::administrator_lifecycle_repository::suspend_administrator as suspend,
};

pub async fn suspend_administrator(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    suspend(
        pool,
        actor_id,
        session_id,
        target_id,
        reason,
        correlation_id,
    )
    .await
}
