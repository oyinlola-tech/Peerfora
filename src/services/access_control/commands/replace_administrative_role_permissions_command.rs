use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::admin_role_repository,
    services::access_control::validation::validate_permission_keys,
};

pub async fn replace_administrative_role_permissions(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    role_id: Uuid,
    permission_keys: &[String],
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    validate_permission_keys(permission_keys)?;
    admin_role_repository::replace_administrative_role_permissions(
        pool,
        actor_id,
        role_id,
        permission_keys,
        correlation_id,
    )
    .await
}
