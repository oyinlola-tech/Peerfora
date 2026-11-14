use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID, ADMIN_SENSITIVE_OPERATION_RECENT_AUTH_MINUTES,
        ADMINISTRATOR_DELETED_AUDIT_ACTION, ADMINISTRATOR_REINSTATED_AUDIT_ACTION,
        ADMINISTRATOR_SUSPENDED_AUDIT_ACTION, ADMINISTRATORS_MANAGE_PERMISSION,
        DELETED_ADMINISTRATOR_EMAIL_DOMAIN,
    },
    enums::{admin_auth::AdministratorLifecycleStatus, admin_auth_error::AdminAuthError},
    errors::app_error::AppError,
};

use super::super::admin_auth_repository::{
    AdministrativeAuditEvent, insert_administrative_audit_event,
};

#[derive(Clone, Copy)]
pub(super) enum AdministratorLifecycleChange {
    Suspend,
    Reinstate,
    Delete,
}

pub(super) async fn change_administrator_status(
    pool: &PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
    change: AdministratorLifecycleChange,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_management(&mut transaction).await?;
    ensure_recent_authentication(&mut transaction, actor_id, session_id).await?;
    ensure_actor_is_active(&mut transaction, actor_id).await?;
    ensure_actor_can_manage_administrators(&mut transaction, actor_id).await?;
    if actor_id == target_id {
        return Err(AdminAuthError::AdministratorSelfManagement.into());
    }
    let target = lock_target_administrator(&mut transaction, target_id).await?;
    if target.is_seeded_root {
        return Err(AdminAuthError::SeededAdministratorProtected.into());
    }
    if !change.can_transition_from(target.status) {
        return Err(AdminAuthError::AdministratorStateConflict.into());
    }
    if target.status == AdministratorLifecycleStatus::Active
        && change.removes_available_super_admin()
    {
        ensure_another_active_super_administrator(&mut transaction, target_id).await?;
    }
    update_target_lifecycle(
        &mut transaction,
        actor_id,
        target_id,
        target.status,
        reason,
        change,
    )
    .await?;
    revoke_target_sessions(&mut transaction, target_id).await?;
    revoke_target_challenges(&mut transaction, target_id).await?;
    revoke_target_password_reset_tokens(&mut transaction, target_id).await?;
    revoke_invitations_created_by_target(&mut transaction, target_id).await?;
    if matches!(change, AdministratorLifecycleChange::Delete) {
        revoke_target_role_assignments(&mut transaction, actor_id, target_id, reason).await?;
    }
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id,
            action: change.audit_action(),
            target_type: "administrator",
            target_id,
            result: "success",
            reason: Some(reason),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_target_password_reset_tokens(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_password_reset_tokens SET revoked_at = now()
        WHERE administrator_id = $1 AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

impl AdministratorLifecycleChange {
    fn next_status(self) -> AdministratorLifecycleStatus {
        match self {
            Self::Suspend => AdministratorLifecycleStatus::Suspended,
            Self::Reinstate => AdministratorLifecycleStatus::Active,
            Self::Delete => AdministratorLifecycleStatus::Deleted,
        }
    }

    fn can_transition_from(self, current_status: AdministratorLifecycleStatus) -> bool {
        match self {
            Self::Suspend => current_status == AdministratorLifecycleStatus::Active,
            Self::Reinstate => current_status == AdministratorLifecycleStatus::Suspended,
            Self::Delete => matches!(
                current_status,
                AdministratorLifecycleStatus::Active | AdministratorLifecycleStatus::Suspended
            ),
        }
    }

    fn removes_available_super_admin(self) -> bool {
        matches!(self, Self::Suspend | Self::Delete)
    }

    fn audit_action(self) -> &'static str {
        match self {
            Self::Suspend => ADMINISTRATOR_SUSPENDED_AUDIT_ACTION,
            Self::Reinstate => ADMINISTRATOR_REINSTATED_AUDIT_ACTION,
            Self::Delete => ADMINISTRATOR_DELETED_AUDIT_ACTION,
        }
    }
}

struct AdministratorLifecycleRecord {
    status: AdministratorLifecycleStatus,
    is_seeded_root: bool,
}

pub(super) async fn lock_administrator_management(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), AppError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub(super) async fn ensure_recent_authentication(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    session_id: Uuid,
) -> Result<(), AppError> {
    let is_recent: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_sessions
            WHERE id = $1 AND administrator_id = $2 AND revoked_at IS NULL
                AND expires_at > now()
                AND created_at >= now() - ($3 * interval '1 minute')
        )",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(ADMIN_SENSITIVE_OPERATION_RECENT_AUTH_MINUTES as f64)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if is_recent {
        Ok(())
    } else {
        Err(AdminAuthError::RecentAdministratorAuthenticationRequired.into())
    }
}

pub(super) async fn ensure_actor_is_active(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let actor_is_active: bool =
        sqlx::query_scalar("SELECT status = 'active' FROM administrators WHERE id = $1 FOR SHARE")
            .bind(actor_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?
            .unwrap_or(false);
    if actor_is_active {
        Ok(())
    } else {
        Err(AdminAuthError::InvalidSession.into())
    }
}

pub(super) async fn ensure_actor_can_manage_administrators(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let has_permission: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles assignment
            JOIN roles role ON role.role_key = assignment.role_key AND role.is_active
            JOIN role_permissions role_permission ON role_permission.role_key = role.role_key
            WHERE assignment.administrator_id = $1
                AND assignment.revoked_at IS NULL
                AND role_permission.permission_key = $2
        )",
    )
    .bind(actor_id)
    .bind(ADMINISTRATORS_MANAGE_PERMISSION)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if has_permission {
        Ok(())
    } else {
        Err(AdminAuthError::PermissionDenied.into())
    }
}

async fn lock_target_administrator(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<AdministratorLifecycleRecord, AppError> {
    let (status, is_seeded_root) = sqlx::query_as::<_, (String, bool)>(
        "SELECT status, is_seeded_root FROM administrators WHERE id = $1 FOR UPDATE",
    )
    .bind(target_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AdminAuthError::AdministratorNotFound)?;
    Ok(AdministratorLifecycleRecord {
        status: AdministratorLifecycleStatus::try_from(status.as_str())
            .map_err(|_| AppError::Internal)?,
        is_seeded_root,
    })
}

async fn ensure_another_active_super_administrator(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    let another_active_super_admin_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles assignment
            JOIN administrators administrator ON administrator.id = assignment.administrator_id
            WHERE assignment.role_key = 'super_administrator'
                AND assignment.revoked_at IS NULL
                AND assignment.administrator_id <> $1
                AND administrator.status = 'active'
                AND administrator.mfa_enabled_at IS NOT NULL
        )",
    )
    .bind(target_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if another_active_super_admin_exists {
        Ok(())
    } else {
        Err(AdminAuthError::LastSuperAdministratorProtected.into())
    }
}

async fn update_target_lifecycle(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    target_id: Uuid,
    current_status: AdministratorLifecycleStatus,
    reason: &str,
    change: AdministratorLifecycleChange,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrators SET status = $1,
            suspended_at = CASE WHEN $1 = 'suspended' THEN now() ELSE NULL END,
            deleted_at = CASE WHEN $1 = 'deleted' THEN now() ELSE NULL END,
            lifecycle_changed_by = $2,
            lifecycle_reason = $3,
            email = CASE WHEN $1 = 'deleted'
                THEN 'deleted-' || id::text || '@' || $6 ELSE email END,
            password_hash = CASE WHEN $1 = 'deleted' THEN NULL ELSE password_hash END,
            mfa_secret_ciphertext = CASE WHEN $1 = 'deleted'
                THEN NULL ELSE mfa_secret_ciphertext END,
            mfa_enabled_at = CASE WHEN $1 = 'deleted' THEN NULL ELSE mfa_enabled_at END,
            last_totp_step = CASE WHEN $1 = 'deleted' THEN NULL ELSE last_totp_step END,
            updated_at = now()
        WHERE id = $4 AND status = $5",
    )
    .bind(change.next_status().as_database_value())
    .bind(actor_id)
    .bind(reason)
    .bind(target_id)
    .bind(current_status.as_database_value())
    .bind(DELETED_ADMINISTRATOR_EMAIL_DOMAIN)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_target_sessions(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_sessions SET revoked_at = now()
        WHERE administrator_id = $1 AND revoked_at IS NULL",
    )
    .bind(target_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_target_challenges(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_mfa_challenges SET consumed_at = now()
        WHERE administrator_id = $1 AND consumed_at IS NULL",
    )
    .bind(target_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_target_role_assignments(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    target_id: Uuid,
    reason: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_roles SET revoked_at = now(), revoked_by = $1,
            revocation_reason = $2
        WHERE administrator_id = $3 AND revoked_at IS NULL",
    )
    .bind(actor_id)
    .bind(reason)
    .bind(target_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_invitations_created_by_target(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_invitations SET revoked_at = now()
        WHERE invited_by = $1 AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(target_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
