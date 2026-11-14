use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMINISTRATOR_ROLE_REPLACEMENT_REASON,
    enums::admin_auth_error::AdminAuthError, errors::app_error::AppError,
    models::admin_role::AdministrativeRoleRecord,
};

use super::admin_auth_repository::{AdministrativeAuditEvent, insert_administrative_audit_event};
use super::admin_role_repository::lock_role_authorization;

pub async fn list_roles_for_administrator(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<Vec<AdministrativeRoleRecord>, AppError> {
    sqlx::query_as::<_, AdministrativeRoleRecord>(
        "SELECT role.id, role.role_key, role.display_name, role.description,
            role.is_system_role, role.is_active,
            COALESCE(array_agg(role_permission.permission_key)
                FILTER (WHERE role_permission.permission_key IS NOT NULL), ARRAY[]::text[])
                AS permissions
        FROM administrator_roles assignment
        JOIN administrators ON administrators.id = assignment.administrator_id
            AND administrators.status = 'active'
        JOIN roles role ON role.role_key = assignment.role_key
        LEFT JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
        WHERE assignment.administrator_id = $1 AND assignment.revoked_at IS NULL
        GROUP BY role.id, role.role_key, role.display_name, role.description,
            role.is_system_role, role.is_active
        ORDER BY role.display_name, role.role_key",
    )
    .bind(administrator_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_permissions_for_administrator(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<Vec<String>, AppError> {
    sqlx::query_scalar(
        "SELECT DISTINCT role_permission.permission_key
        FROM administrator_roles assignment
        JOIN administrators ON administrators.id = assignment.administrator_id
            AND administrators.status = 'active'
        JOIN roles role ON role.role_key = assignment.role_key AND role.is_active
        JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
        WHERE assignment.administrator_id = $1 AND assignment.revoked_at IS NULL
        ORDER BY role_permission.permission_key",
    )
    .bind(administrator_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn replace_administrator_roles(
    pool: &PgPool,
    actor_id: Uuid,
    target_administrator_id: Uuid,
    role_ids: &[Uuid],
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_role_authorization(&mut transaction).await?;
    let (is_seeded_root, target_status) =
        lock_target_administrator(&mut transaction, target_administrator_id).await?;
    if is_seeded_root {
        return Err(AdminAuthError::SeededAdministratorProtected.into());
    }
    if target_status == "deleted" {
        return Err(AdminAuthError::AdministratorStateConflict.into());
    }
    let role_keys = find_active_roles(&mut transaction, role_ids).await?;
    validate_role_permissions_are_delegable(&mut transaction, actor_id, role_ids).await?;
    sqlx::query(
        "UPDATE administrator_roles SET revoked_at = now(), revoked_by = $2,
            revocation_reason = $3
        WHERE administrator_id = $1 AND revoked_at IS NULL",
    )
    .bind(target_administrator_id)
    .bind(actor_id)
    .bind(ADMINISTRATOR_ROLE_REPLACEMENT_REASON)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    for role_key in &role_keys {
        sqlx::query(
            "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
            VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(target_administrator_id)
        .bind(role_key)
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id,
            action: "administrator.roles_replaced",
            target_type: "administrator",
            target_id: target_administrator_id,
            result: "success",
            reason: Some("Assigned roles replaced through authorized delegation"),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn lock_target_administrator(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
) -> Result<(bool, String), AppError> {
    sqlx::query_as::<_, (bool, String)>(
        "SELECT is_seeded_root, status FROM administrators WHERE id = $1 FOR UPDATE",
    )
    .bind(administrator_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or_else(|| AdminAuthError::AdministratorNotFound.into())
}

async fn find_active_roles(
    transaction: &mut Transaction<'_, Postgres>,
    role_ids: &[Uuid],
) -> Result<Vec<String>, AppError> {
    let keys: Vec<String> = sqlx::query_scalar(
        "SELECT role_key FROM roles WHERE id = ANY($1) AND is_active ORDER BY role_key",
    )
    .bind(role_ids)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if keys.len() != role_ids.len() {
        return Err(AdminAuthError::RoleNotFound.into());
    }
    Ok(keys)
}

async fn validate_role_permissions_are_delegable(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    role_ids: &[Uuid],
) -> Result<(), AppError> {
    let actor_permissions: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT role_permission.permission_key
        FROM administrator_roles assignment
        JOIN administrators ON administrators.id = assignment.administrator_id
            AND administrators.status = 'active'
        JOIN roles role ON role.role_key = assignment.role_key AND role.is_active
        JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
        WHERE assignment.administrator_id = $1 AND assignment.revoked_at IS NULL",
    )
    .bind(actor_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let unauthorized_permission_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM roles role
            JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
            WHERE role.id = ANY($1) AND role.is_active
                AND NOT (role_permission.permission_key = ANY($2))
        )",
    )
    .bind(role_ids)
    .bind(actor_permissions)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if unauthorized_permission_exists {
        return Err(AdminAuthError::InvalidPermissionSelection.into());
    }
    Ok(())
}

pub async fn validate_role_key_is_delegable(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    role_key: &str,
) -> Result<(), AppError> {
    let actor_permissions = effective_permission_keys(transaction, actor_id).await?;
    let permission_outside_scope: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM role_permissions
            WHERE role_key = $1
                AND NOT (permission_key = ANY($2))
        )",
    )
    .bind(role_key)
    .bind(actor_permissions)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if permission_outside_scope {
        return Err(AdminAuthError::InvalidPermissionSelection.into());
    }
    Ok(())
}

async fn effective_permission_keys(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
) -> Result<Vec<String>, AppError> {
    sqlx::query_scalar(
        "SELECT DISTINCT role_permission.permission_key
        FROM administrator_roles assignment
        JOIN administrators ON administrators.id = assignment.administrator_id
            AND administrators.status = 'active'
        JOIN roles role ON role.role_key = assignment.role_key AND role.is_active
        JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
        WHERE assignment.administrator_id = $1 AND assignment.revoked_at IS NULL",
    )
    .bind(administrator_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}
