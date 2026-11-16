use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_APPLICATION_AUDIT_TARGET_TYPE, DEVELOPER_APPLICATION_SUSPEND_PERMISSION,
        DEVELOPER_APPLICATION_SUSPENDED_AUDIT_ACTION, DEVELOPER_CREDENTIAL_AUDIT_TARGET_TYPE,
        DEVELOPER_CREDENTIAL_REVOKE_PERMISSION, DEVELOPER_CREDENTIAL_REVOKED_AUDIT_ACTION,
    },
    enums::developer::DeveloperError,
    errors::app_error::AppError,
};

pub async fn suspend_developer_application(
    pool: &PgPool,
    application_id: Uuid,
    administrator_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_authorization(&mut transaction).await?;
    ensure_administrator_permission(
        &mut transaction,
        administrator_id,
        DEVELOPER_APPLICATION_SUSPEND_PERMISSION,
    )
    .await?;
    let status: Option<String> = sqlx::query_scalar(
        "SELECT application_status FROM developer_applications WHERE id = $1 FOR UPDATE",
    )
    .bind(application_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    match status.as_deref() {
        None => return Err(DeveloperError::ApplicationNotFound.into()),
        Some("approved") => {}
        Some("suspended") => return Err(DeveloperError::ApplicationStatusConflict.into()),
        Some(_) => return Err(DeveloperError::ApplicationStatusConflict.into()),
    }
    sqlx::query(
        "UPDATE developer_applications SET application_status = 'suspended'
         WHERE id = $1",
    )
    .bind(application_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    append_developer_audit_record(
        &mut transaction,
        administrator_id,
        DEVELOPER_APPLICATION_SUSPENDED_AUDIT_ACTION,
        DEVELOPER_APPLICATION_AUDIT_TARGET_TYPE,
        application_id,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn revoke_developer_credential_as_administrator(
    pool: &PgPool,
    application_id: Uuid,
    credential_id: Uuid,
    administrator_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_authorization(&mut transaction).await?;
    ensure_administrator_permission(
        &mut transaction,
        administrator_id,
        DEVELOPER_CREDENTIAL_REVOKE_PERMISSION,
    )
    .await?;
    let revoked = sqlx::query(
        "UPDATE developer_application_credentials
         SET revoked_at = now()
         WHERE id = $1 AND application_id = $2 AND revoked_at IS NULL
             AND expires_at > now()",
    )
    .bind(credential_id)
    .bind(application_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if revoked.rows_affected() == 0 {
        return Err(DeveloperError::CredentialNotActive.into());
    }
    append_developer_audit_record(
        &mut transaction,
        administrator_id,
        DEVELOPER_CREDENTIAL_REVOKED_AUDIT_ACTION,
        DEVELOPER_CREDENTIAL_AUDIT_TARGET_TYPE,
        credential_id,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn lock_administrator_authorization(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), AppError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(crate::constants::admin_auth::ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn ensure_administrator_permission(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    permission_key: &str,
) -> Result<(), AppError> {
    let permitted: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles assignment
            JOIN administrators administrator
                ON administrator.id = assignment.administrator_id
                AND administrator.status = 'active'
            JOIN roles role
                ON role.role_key = assignment.role_key AND role.is_active
            JOIN role_permissions permission
                ON permission.role_key = assignment.role_key
            WHERE assignment.administrator_id = $1
                AND assignment.revoked_at IS NULL
                AND permission.permission_key = $2
        )",
    )
    .bind(administrator_id)
    .bind(permission_key)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if permitted {
        Ok(())
    } else {
        Err(crate::enums::admin_auth_error::AdminAuthError::PermissionDenied.into())
    }
}

async fn append_developer_audit_record(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    action: &str,
    target_type: &str,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id,
             result, reason, correlation_id)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, 'success', $5, $6)",
    )
    .bind(administrator_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(reason)
    .bind(correlation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
