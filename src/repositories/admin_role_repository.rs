use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID,
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    models::admin_role::{
        AdministrativePermissionRecord, AdministrativeRoleRecord, NewAdministrativeRole,
    },
};

use super::admin_auth_repository::{AdministrativeAuditEvent, insert_administrative_audit_event};

pub async fn list_administrative_roles(
    pool: &PgPool,
) -> Result<Vec<AdministrativeRoleRecord>, AppError> {
    sqlx::query_as::<_, AdministrativeRoleRecord>(
        "SELECT role.id, role.role_key, role.display_name, role.description,
            role.is_system_role, role.is_active,
            COALESCE(array_agg(role_permission.permission_key)
                FILTER (WHERE role_permission.permission_key IS NOT NULL), ARRAY[]::text[])
                AS permissions
        FROM roles role
        LEFT JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
        WHERE role.deleted_at IS NULL
        GROUP BY role.id, role.role_key, role.display_name, role.description,
            role.is_system_role, role.is_active
        ORDER BY role.is_system_role DESC, role.display_name, role.role_key",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn find_administrative_role(
    pool: &PgPool,
    role_id: Uuid,
) -> Result<Option<AdministrativeRoleRecord>, AppError> {
    sqlx::query_as::<_, AdministrativeRoleRecord>(
        "SELECT role.id, role.role_key, role.display_name, role.description,
            role.is_system_role, role.is_active,
            COALESCE(array_agg(role_permission.permission_key)
                FILTER (WHERE role_permission.permission_key IS NOT NULL), ARRAY[]::text[])
                AS permissions
        FROM roles role
        LEFT JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
        WHERE role.id = $1 AND role.deleted_at IS NULL
        GROUP BY role.id, role.role_key, role.display_name, role.description,
            role.is_system_role, role.is_active",
    )
    .bind(role_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_administrative_permissions(
    pool: &PgPool,
) -> Result<Vec<AdministrativePermissionRecord>, AppError> {
    sqlx::query_as::<_, AdministrativePermissionRecord>(
        "SELECT permission_key, description, is_assignable, is_sensitive
        FROM permissions ORDER BY permission_key",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn create_administrative_role(
    pool: &PgPool,
    actor_id: Uuid,
    role: NewAdministrativeRole<'_>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_role_authorization(&mut transaction).await?;
    validate_delegated_permissions(&mut transaction, actor_id, role.permission_keys).await?;
    let inserted = sqlx::query(
        "INSERT INTO roles (id, role_key, display_name, description, is_system_role, created_by)
        VALUES ($1, $2, $3, $4, FALSE, $5)
        ON CONFLICT (role_key) DO NOTHING",
    )
    .bind(role.id)
    .bind(role.role_key)
    .bind(role.display_name)
    .bind(role.description)
    .bind(actor_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if inserted.rows_affected() != 1 {
        return Err(AdminAuthError::RoleKeyAlreadyExists.into());
    }
    replace_role_permissions_in_transaction(
        &mut transaction,
        actor_id,
        role.id,
        role.permission_keys,
    )
    .await?;
    record_role_audit(
        &mut transaction,
        actor_id,
        role.id,
        "administrator.role_created",
        Some(role.role_key),
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn update_administrative_role(
    pool: &PgPool,
    actor_id: Uuid,
    role_id: Uuid,
    display_name: Option<&str>,
    description: Option<&str>,
    is_active: Option<bool>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_role_authorization(&mut transaction).await?;
    let result = sqlx::query(
        "UPDATE roles SET
            display_name = COALESCE($2, display_name),
            description = COALESCE($3, description),
            is_active = COALESCE($4, is_active), updated_at = now()
        WHERE id = $1 AND NOT is_system_role AND deleted_at IS NULL",
    )
    .bind(role_id)
    .bind(display_name)
    .bind(description)
    .bind(is_active)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() == 0 {
        ensure_custom_role_exists(&mut transaction, role_id).await?;
        return Err(AdminAuthError::SystemRoleProtected.into());
    }
    record_role_audit(
        &mut transaction,
        actor_id,
        role_id,
        "administrator.role_updated",
        None,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn delete_administrative_role(
    pool: &PgPool,
    actor_id: Uuid,
    role_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_role_authorization(&mut transaction).await?;
    let role_key = find_custom_role_key(&mut transaction, role_id).await?;
    let assigned: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles
            WHERE role_key = $1 AND revoked_at IS NULL
            UNION ALL
            SELECT 1 FROM administrator_invitations
            WHERE role_key = $1 AND consumed_at IS NULL AND revoked_at IS NULL
                AND expires_at > now()
        )",
    )
    .bind(&role_key)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if assigned {
        return Err(AdminAuthError::RoleInUse.into());
    }
    sqlx::query(
        "UPDATE roles SET is_active = FALSE, deleted_at = now(), updated_at = now()
        WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(role_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    record_role_audit(
        &mut transaction,
        actor_id,
        role_id,
        "administrator.role_deleted",
        Some(&role_key),
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn replace_administrative_role_permissions(
    pool: &PgPool,
    actor_id: Uuid,
    role_id: Uuid,
    permission_keys: &[String],
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_role_authorization(&mut transaction).await?;
    let role_key = find_custom_role_key(&mut transaction, role_id).await?;
    replace_role_permissions_in_transaction(&mut transaction, actor_id, role_id, permission_keys)
        .await?;
    record_role_audit(
        &mut transaction,
        actor_id,
        role_id,
        "administrator.role_permissions_replaced",
        Some(&role_key),
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn replace_role_permissions_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    role_id: Uuid,
    permission_keys: &[String],
) -> Result<(), AppError> {
    validate_delegated_permissions(transaction, actor_id, permission_keys).await?;
    let role_key = find_custom_role_key(transaction, role_id).await?;
    sqlx::query("DELETE FROM role_permissions WHERE role_key = $1")
        .bind(&role_key)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    for permission_key in permission_keys {
        sqlx::query(
            "INSERT INTO role_permissions (role_key, permission_key)
            VALUES ($1, $2)",
        )
        .bind(&role_key)
        .bind(permission_key)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}

async fn validate_delegated_permissions(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    permission_keys: &[String],
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
    let permitted_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM permissions
        WHERE permission_key = ANY($1) AND is_assignable
            AND permission_key = ANY($2)",
    )
    .bind(permission_keys)
    .bind(&actor_permissions)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if usize::try_from(permitted_count).ok() != Some(permission_keys.len()) {
        return Err(AdminAuthError::InvalidPermissionSelection.into());
    }
    Ok(())
}

pub(super) async fn lock_role_authorization(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), AppError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn find_custom_role_key(
    transaction: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
) -> Result<String, AppError> {
    let (role_key, is_system_role): (String, bool) = sqlx::query_as(
        "SELECT role_key, is_system_role FROM roles
            WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(role_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AdminAuthError::RoleNotFound)?;
    if is_system_role {
        return Err(AdminAuthError::SystemRoleProtected.into());
    }
    Ok(role_key)
}

async fn ensure_custom_role_exists(
    transaction: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
) -> Result<(), AppError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM roles WHERE id = $1)")
        .bind(role_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    if !exists {
        return Err(AdminAuthError::RoleNotFound.into());
    }
    Ok(())
}

async fn record_role_audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    role_id: Uuid,
    action: &'static str,
    reason: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    insert_administrative_audit_event(
        transaction,
        AdministrativeAuditEvent {
            actor_id,
            action,
            target_type: "administrator_role",
            target_id: role_id,
            result: "success",
            reason,
            correlation_id,
        },
    )
    .await
}
