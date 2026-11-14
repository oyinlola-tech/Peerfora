use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    repositories::administrator_lifecycle_repository::reinstate_administrator as reinstate,
};

pub async fn reinstate_administrator(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    reinstate(
        pool,
        actor_id,
        session_id,
        target_id,
        reason,
        correlation_id,
    )
    .await
}
