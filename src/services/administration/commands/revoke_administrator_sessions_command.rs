use uuid::Uuid;

use crate::{
    enums::administrator_lifecycle_reason::AdministratorLifecycleReason,
    errors::app_error::AppError,
    repositories::administrator_lifecycle_repository::revoke_administrator_sessions as revoke,
};

pub async fn revoke_administrator_sessions(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: AdministratorLifecycleReason,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    revoke(
        pool,
        actor_id,
        session_id,
        target_id,
        reason.as_str(),
        correlation_id,
    )
    .await
}
