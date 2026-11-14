use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    repositories::administrator_lifecycle_repository::delete_administrator as delete,
};

pub async fn delete_administrator(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    delete(
        pool,
        actor_id,
        session_id,
        target_id,
        reason,
        correlation_id,
    )
    .await
}
