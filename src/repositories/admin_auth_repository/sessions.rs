use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMIN_SESSION_IDLE_TIMEOUT_MINUTES,
    errors::app_error::AppError,
    models::admin_identity::{
        AdministrativeAuditLogRecord, AdministratorSessionIdentity, AdministratorSessionRecord,
    },
};

use super::audit::{AdministrativeAuditEvent, insert_administrative_audit_event};

pub async fn find_active_administrator_session(
    pool: &PgPool,
    session_token_hash: &str,
) -> Result<Option<AdministratorSessionIdentity>, AppError> {
    let identity = sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT session.id, session.administrator_id
        FROM administrator_sessions session
        JOIN administrators administrator ON administrator.id = session.administrator_id
        WHERE session.token_hash = $1 AND session.revoked_at IS NULL
            AND session.expires_at > now()
            AND session.last_activity_at > now() - ($2 * interval '1 minute')
            AND administrator.status = 'active' AND administrator.mfa_enabled_at IS NOT NULL",
    )
    .bind(session_token_hash)
    .bind(ADMIN_SESSION_IDLE_TIMEOUT_MINUTES as f64)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some((session_id, administrator_id)) = identity else {
        return Ok(None);
    };
    sqlx::query("UPDATE administrator_sessions SET last_activity_at = now() WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Some(AdministratorSessionIdentity {
        administrator_id,
        session_id,
    }))
}

pub async fn find_administrator_session_profile(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<(String, Vec<String>, Vec<String>), AppError> {
    sqlx::query_as::<_, (String, Vec<String>, Vec<String>)>(
        "SELECT administrator.email,
            ARRAY(SELECT administrator_roles.role_key FROM administrator_roles
                JOIN administrators assignment_owner
                    ON assignment_owner.id = administrator_roles.administrator_id
                    AND assignment_owner.status = 'active'
                JOIN roles ON roles.role_key = administrator_roles.role_key AND roles.is_active
                WHERE administrator_roles.administrator_id = administrator.id
                    AND administrator_roles.revoked_at IS NULL),
            ARRAY(SELECT DISTINCT role_permissions.permission_key
                FROM administrator_roles
                JOIN administrators assignment_owner
                    ON assignment_owner.id = administrator_roles.administrator_id
                    AND assignment_owner.status = 'active'
                JOIN roles ON roles.role_key = administrator_roles.role_key AND roles.is_active
                JOIN role_permissions
                    ON role_permissions.role_key = administrator_roles.role_key
                WHERE administrator_roles.administrator_id = administrator.id
                    AND administrator_roles.revoked_at IS NULL)
        FROM administrators administrator
        WHERE administrator.id = $1 AND administrator.status = 'active'",
    )
    .bind(administrator_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidSession.into())
}

pub async fn administrator_has_permission(
    pool: &PgPool,
    administrator_id: Uuid,
    permission_key: &str,
) -> Result<bool, AppError> {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles
            JOIN administrators ON administrators.id = administrator_roles.administrator_id
                AND administrators.status = 'active'
            JOIN roles ON roles.role_key = administrator_roles.role_key AND roles.is_active
            JOIN role_permissions
                ON role_permissions.role_key = administrator_roles.role_key
            WHERE administrator_roles.administrator_id = $1
                AND role_permissions.permission_key = $2
                AND administrator_roles.revoked_at IS NULL
        )",
    )
    .bind(administrator_id)
    .bind(permission_key)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn record_administrator_permission_denial(
    pool: &PgPool,
    administrator_id: Uuid,
    permission_key: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.permission_denied",
            target_type: "permission",
            target_id: Uuid::nil(),
            result: "failure",
            reason: Some(permission_key),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_administrative_audit_logs(
    pool: &PgPool,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AdministrativeAuditLogRecord>, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.audit_logs_viewed",
            target_type: "administrative_audit_logs",
            target_id: Uuid::nil(),
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    let records = sqlx::query_as::<_, AdministrativeAuditLogRecord>(
        "SELECT id, actor_administrator_id, action, target_type, target_id,
            result, reason, correlation_id, created_at
        FROM administrative_audit_logs
        ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(records)
}

pub async fn revoke_administrator_session(
    pool: &PgPool,
    administrator_id: Uuid,
    session_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let revoked = sqlx::query(
        "UPDATE administrator_sessions SET revoked_at = now()
        WHERE id = $1 AND administrator_id = $2 AND revoked_at IS NULL",
    )
    .bind(session_id)
    .bind(administrator_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if revoked.rows_affected() != 1 {
        return Err(crate::enums::admin_auth_error::AdminAuthError::SessionNotFound.into());
    }
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.session_revoked",
            target_type: "administrator_session",
            target_id: session_id,
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_administrator_sessions(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<Vec<AdministratorSessionRecord>, AppError> {
    sqlx::query_as::<_, AdministratorSessionRecord>(
        "SELECT id, created_at, last_activity_at, expires_at, revoked_at
        FROM administrator_sessions WHERE administrator_id = $1
        ORDER BY created_at DESC, id DESC LIMIT 50",
    )
    .bind(administrator_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
