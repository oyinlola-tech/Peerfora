use crate::{errors::app_error::AppError, models::admin_role::AdministrativePermissionRecord};

pub async fn list_administrative_permissions(
    pool: &sqlx::PgPool,
) -> Result<Vec<AdministrativePermissionRecord>, AppError> {
    crate::repositories::admin_role_repository::list_administrative_permissions(pool).await
}
