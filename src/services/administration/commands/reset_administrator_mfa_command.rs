use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::administrator_lifecycle_repository};

pub async fn reset_administrator_mfa(
    pool: &PgPool,
    actor_id: Uuid,
    actor_session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    administrator_lifecycle_repository::reset_administrator_mfa(
        pool,
        actor_id,
        actor_session_id,
        target_id,
        reason,
        correlation_id,
    )
    .await
}
