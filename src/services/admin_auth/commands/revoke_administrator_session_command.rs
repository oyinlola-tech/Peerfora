use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::admin_auth_repository};

pub async fn revoke_administrator_session(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
    session_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    admin_auth_repository::revoke_administrator_session(
        pool,
        administrator_id,
        session_id,
        correlation_id,
    )
    .await
}
