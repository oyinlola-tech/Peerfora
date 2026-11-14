use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::admin_role_repository};

pub async fn delete_administrative_role(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    role_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    admin_role_repository::delete_administrative_role(pool, actor_id, role_id, correlation_id).await
}
