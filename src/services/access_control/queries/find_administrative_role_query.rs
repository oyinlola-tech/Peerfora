use uuid::Uuid;

use crate::{
    enums::admin_auth_error::AdminAuthError, errors::app_error::AppError,
    models::admin_role::AdministrativeRoleRecord,
};

pub async fn find_administrative_role(
    pool: &sqlx::PgPool,
    role_id: Uuid,
) -> Result<AdministrativeRoleRecord, AppError> {
    crate::repositories::admin_role_repository::find_administrative_role(pool, role_id)
        .await?
        .ok_or_else(|| AdminAuthError::RoleNotFound.into())
}
