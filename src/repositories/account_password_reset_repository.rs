use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::account_auth::PASSWORD_RESET_LIFETIME_HOURS,
    enums::account_auth_error::AccountAuthError, errors::app_error::AppError,
};

pub async fn create_password_reset_token(
    pool: &PgPool,
    email: &str,
    token_hash: &str,
) -> Result<Option<String>, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let account = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, email FROM accounts
         WHERE lower(email) = lower($1) AND account_status = 'active' FOR UPDATE",
    )
    .bind(email)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some((account_id, recipient)) = account else {
        return Ok(None);
    };
    sqlx::query("DELETE FROM password_reset_tokens WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "INSERT INTO password_reset_tokens (account_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(account_id)
    .bind(token_hash)
    .bind(Utc::now() + Duration::hours(PASSWORD_RESET_LIFETIME_HOURS))
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(Some(recipient))
}

pub async fn reset_account_password(
    pool: &PgPool,
    token_hash: &str,
    password_hash: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let account_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT account.id FROM accounts account
         JOIN password_reset_tokens token ON token.account_id = account.id
         WHERE token.token_hash = $1 AND token.expires_at > now()
             AND account.account_status = 'active'
         FOR UPDATE OF account",
    )
    .bind(token_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AccountAuthError::PasswordResetInvalid)?;
    let consumed = sqlx::query(
        "DELETE FROM password_reset_tokens
         WHERE account_id = $1 AND token_hash = $2 AND expires_at > now()",
    )
    .bind(account_id)
    .bind(token_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if consumed.rows_affected() != 1 {
        return Err(AccountAuthError::PasswordResetInvalid.into());
    }
    sqlx::query("UPDATE accounts SET password_hash = $1, updated_at = now() WHERE id = $2")
        .bind(password_hash)
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM account_sessions WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM password_reset_tokens WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}
