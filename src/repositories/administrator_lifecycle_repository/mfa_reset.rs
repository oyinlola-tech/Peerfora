use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMINISTRATOR_MFA_RESET_AUDIT_ACTION,
    enums::admin_auth_error::AdminAuthError, errors::app_error::AppError,
};

use super::{
    super::admin_auth_repository::{AdministrativeAuditEvent, insert_administrative_audit_event},
    status_transition::{
        ensure_actor_can_manage_administrators, ensure_actor_is_active,
        ensure_recent_authentication, lock_administrator_management,
    },
};

pub async fn reset_administrator_mfa(
    pool: &PgPool,
    actor_id: Uuid,
    actor_session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_management(&mut transaction).await?;
    ensure_recent_authentication(&mut transaction, actor_id, actor_session_id).await?;
    ensure_actor_is_active(&mut transaction, actor_id).await?;
    ensure_actor_can_manage_administrators(&mut transaction, actor_id).await?;
    reject_self_mfa_reset(actor_id, target_id)?;
    lock_active_mfa_reset_target(&mut transaction, target_id).await?;
    clear_target_mfa_credential(&mut transaction, target_id).await?;
    revoke_target_mfa_sessions(&mut transaction, target_id).await?;
    consume_target_mfa_challenges(&mut transaction, target_id).await?;
    revoke_unused_target_recovery_codes(&mut transaction, target_id).await?;
    record_mfa_reset(
        &mut transaction,
        actor_id,
        target_id,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

fn reject_self_mfa_reset(actor_id: Uuid, target_id: Uuid) -> Result<(), AppError> {
    if actor_id == target_id {
        Err(AdminAuthError::AdministratorSelfMfaReset.into())
    } else {
        Ok(())
    }
}

async fn lock_active_mfa_reset_target(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    let active_target = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM administrators WHERE id = $1 AND status = 'active' FOR UPDATE",
    )
    .bind(target_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_target.is_some() {
        Ok(())
    } else {
        Err(AdminAuthError::AdministratorNotFound.into())
    }
}

async fn clear_target_mfa_credential(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrators SET mfa_secret_ciphertext = NULL,
            mfa_enabled_at = NULL, last_totp_step = NULL,
            failed_mfa_attempts = 0, mfa_locked_until = NULL, updated_at = now()
        WHERE id = $1 AND status = 'active'",
    )
    .bind(target_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_target_mfa_sessions(
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

async fn consume_target_mfa_challenges(
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

async fn revoke_unused_target_recovery_codes(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_recovery_codes SET revoked_at = now()
        WHERE administrator_id = $1 AND used_at IS NULL AND revoked_at IS NULL",
    )
    .bind(target_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn record_mfa_reset(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    insert_administrative_audit_event(
        transaction,
        AdministrativeAuditEvent {
            actor_id,
            action: ADMINISTRATOR_MFA_RESET_AUDIT_ACTION,
            target_type: "administrator",
            target_id,
            result: "success",
            reason: Some(reason),
            correlation_id,
        },
    )
    .await
}
