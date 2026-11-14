use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMINISTRATOR_SESSIONS_REVOKED_AUDIT_ACTION,
    enums::admin_auth_error::AdminAuthError, errors::app_error::AppError,
};

use super::{
    super::admin_auth_repository::{AdministrativeAuditEvent, insert_administrative_audit_event},
    status_transition::{
        ensure_actor_can_manage_administrators, ensure_actor_is_active,
        ensure_recent_authentication, lock_administrator_management,
    },
};

pub async fn revoke_administrator_sessions(
    pool: &PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_management(&mut transaction).await?;
    ensure_recent_authentication(&mut transaction, actor_id, session_id).await?;
    ensure_actor_is_active(&mut transaction, actor_id).await?;
    ensure_actor_can_manage_administrators(&mut transaction, actor_id).await?;
    reject_self_session_revocation(actor_id, target_id)?;
    lock_session_owner(&mut transaction, target_id).await?;
    consume_pending_administrator_mfa_challenges(&mut transaction, target_id).await?;
    mark_active_administrator_sessions_revoked(&mut transaction, target_id).await?;
    record_administrator_session_revocation(
        &mut transaction,
        actor_id,
        target_id,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn consume_pending_administrator_mfa_challenges(
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

fn reject_self_session_revocation(actor_id: Uuid, target_id: Uuid) -> Result<(), AppError> {
    if actor_id == target_id {
        Err(AdminAuthError::AdministratorSelfSessionRevocation.into())
    } else {
        Ok(())
    }
}

async fn lock_session_owner(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
) -> Result<(), AppError> {
    let administrator_id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM administrators WHERE id = $1 FOR NO KEY UPDATE")
            .bind(target_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    if administrator_id.is_some() {
        Ok(())
    } else {
        Err(AdminAuthError::AdministratorNotFound.into())
    }
}

async fn mark_active_administrator_sessions_revoked(
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

async fn record_administrator_session_revocation(
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
            action: ADMINISTRATOR_SESSIONS_REVOKED_AUDIT_ACTION,
            target_type: "administrator",
            target_id,
            result: "success",
            reason: Some(reason),
            correlation_id,
        },
    )
    .await
}
