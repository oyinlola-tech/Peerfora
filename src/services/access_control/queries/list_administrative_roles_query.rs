use crate::{errors::app_error::AppError, models::admin_role::AdministrativeRoleRecord};

pub async fn list_administrative_roles(
    pool: &sqlx::PgPool,
) -> Result<Vec<AdministrativeRoleRecord>, AppError> {
    crate::repositories::admin_role_repository::list_administrative_roles(pool).await
}
