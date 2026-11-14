use uuid::Uuid;

use crate::{
    enums::admin_auth_error::AdminAuthError, errors::app_error::AppError,
    models::admin_role::AdministrativeRoleRecord, repositories::admin_role_assignment_repository,
};

pub async fn list_administrator_role_assignments(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
) -> Result<Vec<AdministrativeRoleRecord>, AppError> {
    ensure_administrator_exists(pool, administrator_id).await?;
    admin_role_assignment_repository::list_roles_for_administrator(pool, administrator_id).await
}

pub async fn list_administrator_permission_keys(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
) -> Result<Vec<String>, AppError> {
    ensure_administrator_exists(pool, administrator_id).await?;
    admin_role_assignment_repository::list_permissions_for_administrator(pool, administrator_id)
        .await
}

async fn ensure_administrator_exists(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM administrators WHERE id = $1)")
            .bind(administrator_id)
            .fetch_one(pool)
            .await
            .map_err(|_| AppError::Internal)?;
    if exists {
        Ok(())
    } else {
        Err(AdminAuthError::AdministratorNotFound.into())
    }
}
