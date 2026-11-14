use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_LOGIN_FAILED_ATTEMPT_LIMIT, ADMIN_LOGIN_LOCK_MINUTES,
        ADMIN_SESSION_IDLE_TIMEOUT_MINUTES, ADMINISTRATOR_PASSWORD_CHANGE_FAILED_AUDIT_ACTION,
        ADMINISTRATOR_PASSWORD_CHANGED_AUDIT_ACTION,
    },
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
};

use super::audit::{AdministrativeAuditEvent, insert_administrative_audit_event};

pub async fn find_active_administrator_password_hash(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<String, AppError> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT password_hash FROM administrators
        WHERE id = $1 AND status = 'active'",
    )
    .bind(administrator_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .flatten()
    .ok_or_else(|| AdminAuthError::InvalidSession.into())
}

pub async fn record_failed_administrator_password_change(
    pool: &PgPool,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE administrators SET
            failed_login_attempts = LEAST(failed_login_attempts + 1, $2),
            login_locked_until = CASE
                WHEN failed_login_attempts + 1 >= $2
                    THEN now() + ($3 * interval '1 minute')
                ELSE login_locked_until END,
            updated_at = now()
        WHERE id = $1",
    )
    .bind(administrator_id)
    .bind(ADMIN_LOGIN_FAILED_ATTEMPT_LIMIT)
    .bind(ADMIN_LOGIN_LOCK_MINUTES as f64)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: ADMINISTRATOR_PASSWORD_CHANGE_FAILED_AUDIT_ACTION,
            target_type: "administrator",
            target_id: administrator_id,
            result: "failure",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn change_administrator_password(
    pool: &PgPool,
    administrator_id: Uuid,
    session_id: Uuid,
    previous_password_hash: &str,
    new_password_hash: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_active_administrator(&mut transaction, administrator_id, previous_password_hash).await?;
    lock_current_administrator_session(&mut transaction, administrator_id, session_id).await?;
    store_new_administrator_password_hash(&mut transaction, administrator_id, new_password_hash)
        .await?;
    revoke_administrator_password_reset_tokens(&mut transaction, administrator_id).await?;
    revoke_administrator_sessions(&mut transaction, administrator_id).await?;
    consume_administrator_mfa_challenges(&mut transaction, administrator_id).await?;
    record_administrator_password_change(&mut transaction, administrator_id, correlation_id)
        .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn revoke_administrator_password_reset_tokens(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
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

async fn lock_active_administrator(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    administrator_id: Uuid,
    previous_password_hash: &str,
) -> Result<(), AppError> {
    let credentials = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT status, password_hash FROM administrators WHERE id = $1 FOR NO KEY UPDATE",
    )
    .bind(administrator_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AdminAuthError::InvalidSession)?;
    if credentials.0 != "active" {
        return Err(AdminAuthError::InvalidSession.into());
    }
    if credentials.1.as_deref() != Some(previous_password_hash) {
        return Err(AdminAuthError::AdministratorCredentialChanged.into());
    }
    Ok(())
}

async fn lock_current_administrator_session(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    administrator_id: Uuid,
    session_id: Uuid,
) -> Result<(), AppError> {
    let active_session_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM administrator_sessions
        WHERE id = $1 AND administrator_id = $2 AND revoked_at IS NULL
            AND expires_at > now()
            AND last_activity_at > now() - ($3 * interval '1 minute')
        FOR UPDATE",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(ADMIN_SESSION_IDLE_TIMEOUT_MINUTES as f64)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_session_id.is_some() {
        Ok(())
    } else {
        Err(AdminAuthError::InvalidSession.into())
    }
}

async fn store_new_administrator_password_hash(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    administrator_id: Uuid,
    new_password_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrators SET password_hash = $2, failed_login_attempts = 0,
            login_locked_until = NULL, updated_at = now()
        WHERE id = $1 AND status = 'active'",
    )
    .bind(administrator_id)
    .bind(new_password_hash)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_administrator_sessions(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_sessions SET revoked_at = now()
        WHERE administrator_id = $1 AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn consume_administrator_mfa_challenges(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_mfa_challenges SET consumed_at = now()
        WHERE administrator_id = $1 AND consumed_at IS NULL",
    )
    .bind(administrator_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn record_administrator_password_change(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    insert_administrative_audit_event(
        transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: ADMINISTRATOR_PASSWORD_CHANGED_AUDIT_ACTION,
            target_type: "administrator",
            target_id: administrator_id,
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await
}
