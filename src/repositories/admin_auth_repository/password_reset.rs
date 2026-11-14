use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_PASSWORD_RESET_LIFETIME_MINUTES, ADMINISTRATOR_PASSWORD_RESET_COMPLETED_AUDIT_ACTION,
    },
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    models::admin_identity::AdministratorPasswordResetRecipient,
};

use super::audit::{AdministrativeAuditEvent, insert_administrative_audit_event};

pub async fn create_administrator_password_reset_token(
    pool: &PgPool,
    normalized_email: &str,
    token_hash: &str,
) -> Result<Option<AdministratorPasswordResetRecipient>, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let administrator = sqlx::query_as::<_, AdministratorPasswordResetRecipient>(
        "SELECT id AS administrator_id, email FROM administrators
        WHERE lower(email) = lower($1) AND status = 'active'
        FOR NO KEY UPDATE",
    )
    .bind(normalized_email)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some(administrator) = administrator else {
        transaction.commit().await.map_err(|_| AppError::Internal)?;
        return Ok(None);
    };
    revoke_unused_administrator_password_reset_tokens(
        &mut transaction,
        administrator.administrator_id,
    )
    .await?;
    sqlx::query(
        "INSERT INTO administrator_password_reset_tokens
            (token_hash, administrator_id, expires_at)
        VALUES ($1, $2, now() + ($3 * interval '1 minute'))",
    )
    .bind(token_hash)
    .bind(administrator.administrator_id)
    .bind(ADMIN_PASSWORD_RESET_LIFETIME_MINUTES as f64)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(Some(administrator))
}

pub async fn revoke_failed_administrator_password_reset_delivery(
    pool: &PgPool,
    administrator_id: Uuid,
    token_hash: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE administrator_password_reset_tokens SET revoked_at = now()
        WHERE administrator_id = $1 AND token_hash = $2
            AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .bind(token_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn complete_administrator_password_reset(
    pool: &PgPool,
    token_hash: &str,
    new_password_hash: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let administrator_id =
        find_eligible_password_reset_administrator(&mut transaction, token_hash).await?;
    lock_active_password_reset_administrator(&mut transaction, administrator_id).await?;
    lock_eligible_administrator_password_reset_token(
        &mut transaction,
        administrator_id,
        token_hash,
    )
    .await?;
    update_administrator_password(&mut transaction, administrator_id, new_password_hash).await?;
    consume_used_administrator_password_reset_token(&mut transaction, token_hash).await?;
    revoke_unused_administrator_password_reset_tokens(&mut transaction, administrator_id).await?;
    revoke_administrator_sessions_and_mfa_challenges(&mut transaction, administrator_id).await?;
    audit_administrator_password_reset(&mut transaction, administrator_id, correlation_id).await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn find_eligible_password_reset_administrator(
    transaction: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT administrator_id FROM administrator_password_reset_tokens
        WHERE token_hash = $1 AND expires_at > now()
            AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AdminAuthError::InvalidAdministratorPasswordResetToken.into())
}

async fn lock_active_password_reset_administrator(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    let active_administrator_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM administrators
        WHERE id = $1 AND status = 'active' FOR NO KEY UPDATE",
    )
    .bind(administrator_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_administrator_id.is_some() {
        Ok(())
    } else {
        Err(AdminAuthError::InvalidAdministratorPasswordResetToken.into())
    }
}

async fn lock_eligible_administrator_password_reset_token(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    token_hash: &str,
) -> Result<(), AppError> {
    let active_token_hash = sqlx::query_scalar::<_, String>(
        "SELECT token_hash FROM administrator_password_reset_tokens
        WHERE token_hash = $1 AND administrator_id = $2 AND expires_at > now()
            AND consumed_at IS NULL AND revoked_at IS NULL FOR UPDATE",
    )
    .bind(token_hash)
    .bind(administrator_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_token_hash.is_some() {
        Ok(())
    } else {
        Err(AdminAuthError::InvalidAdministratorPasswordResetToken.into())
    }
}

async fn update_administrator_password(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    new_password_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrators SET password_hash = $2,
            failed_login_attempts = 0, login_locked_until = NULL, updated_at = now()
        WHERE id = $1 AND status = 'active'",
    )
    .bind(administrator_id)
    .bind(new_password_hash)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn consume_used_administrator_password_reset_token(
    transaction: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_password_reset_tokens SET consumed_at = now()
        WHERE token_hash = $1 AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_unused_administrator_password_reset_tokens(
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

async fn revoke_administrator_sessions_and_mfa_challenges(
    transaction: &mut Transaction<'_, Postgres>,
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

async fn audit_administrator_password_reset(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    insert_administrative_audit_event(
        transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: ADMINISTRATOR_PASSWORD_RESET_COMPLETED_AUDIT_ACTION,
            target_type: "administrator",
            target_id: administrator_id,
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await
}
