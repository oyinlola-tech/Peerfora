use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::account_auth::{EMAIL_VERIFICATION_LIFETIME_HOURS, PASSWORD_RESET_LIFETIME_HOURS},
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::account::Account,
};

pub struct NewAccountRecord<'a> {
    pub email: &'a str,
    pub username: &'a str,
    pub display_name: Option<&'a str>,
    pub password_hash: &'a str,
    pub country_code: &'a str,
    pub age_policy_version: &'a str,
    pub verification_token_hash: &'a str,
}

pub async fn create_account(
    pool: &PgPool,
    input: NewAccountRecord<'_>,
) -> Result<Account, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let account = sqlx::query_as::<_, Account>(
        "INSERT INTO accounts (
            email, username, display_name, password_hash,
            eligibility_country_code, age_policy_version, terms_accepted_at
        ) VALUES ($1, $2, $3, $4, $5, $6, now())
        RETURNING id, email, username, display_name,
            password_hash, email_verified_at",
    )
    .bind(input.email)
    .bind(input.username)
    .bind(input.display_name)
    .bind(input.password_hash)
    .bind(input.country_code)
    .bind(input.age_policy_version)
    .fetch_one(&mut *transaction)
    .await
    .map_err(map_database_error)?;
    sqlx::query(
        "INSERT INTO email_verification_tokens (account_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(account.id)
    .bind(input.verification_token_hash)
    .bind(Utc::now() + Duration::hours(EMAIL_VERIFICATION_LIFETIME_HOURS))
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(account)
}

pub async fn find_account_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<Account>, AppError> {
    sqlx::query_as::<_, Account>(
        "SELECT id, email, username, display_name, password_hash,
            email_verified_at FROM accounts
        WHERE lower(email) = lower($1) AND account_status = 'active'",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn create_email_verification_token_for_unverified_account(
    pool: &PgPool,
    email: &str,
    token_hash: &str,
) -> Result<Option<String>, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let account = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, email FROM accounts
         WHERE lower(email) = lower($1) AND account_status = 'active'
             AND email_verified_at IS NULL AND password_hash IS NOT NULL
         FOR UPDATE",
    )
    .bind(email)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some((account_id, account_email)) = account else {
        return Ok(None);
    };
    sqlx::query(
        "INSERT INTO email_verification_tokens (account_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(account_id)
    .bind(token_hash)
    .bind(Utc::now() + Duration::hours(EMAIL_VERIFICATION_LIFETIME_HOURS))
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(Some(account_email))
}

pub async fn verify_account_email(pool: &PgPool, token_hash: &str) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let account_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT account.id FROM accounts account
         JOIN email_verification_tokens token ON token.account_id = account.id
         WHERE token.token_hash = $1 AND token.expires_at > now()
             AND account.account_status = 'active' AND account.email_verified_at IS NULL
         FOR UPDATE OF account",
    )
    .bind(token_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AccountAuthError::EmailConfirmationInvalid)?;
    let valid_token = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM email_verification_tokens
         WHERE account_id = $1 AND token_hash = $2 AND expires_at > now())",
    )
    .bind(account_id)
    .bind(token_hash)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !valid_token {
        return Err(AccountAuthError::EmailConfirmationInvalid.into());
    }
    sqlx::query("UPDATE accounts SET email_verified_at = now() WHERE id = $1")
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM email_verification_tokens WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn create_session(
    pool: &PgPool,
    account_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "WITH eligible_account AS (
            SELECT id FROM accounts
            WHERE id = $1 AND account_status = 'active'
            FOR UPDATE
        )
        INSERT INTO account_sessions (account_id, token_hash, expires_at)
        SELECT eligible_account.id, $2, $3 FROM eligible_account",
    )
    .bind(account_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() != 1 {
        return Err(AccountAuthError::SessionInvalid.into());
    }
    Ok(())
}

pub async fn create_password_reset_token(
    pool: &PgPool,
    account_id: Uuid,
    token_hash: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM password_reset_tokens WHERE account_id = $1")
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "INSERT INTO password_reset_tokens (
            account_id, token_hash, expires_at
        ) VALUES ($1, $2, $3)",
    )
    .bind(account_id)
    .bind(token_hash)
    .bind(Utc::now() + Duration::hours(PASSWORD_RESET_LIFETIME_HOURS))
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn reset_account_password(
    pool: &PgPool,
    token_hash: &str,
    password_hash: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let account_id = sqlx::query_scalar::<_, Uuid>(
        "DELETE FROM password_reset_tokens
        WHERE token_hash = $1 AND expires_at > now()
        RETURNING account_id",
    )
    .bind(token_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AccountAuthError::PasswordResetInvalid)?;
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

pub async fn find_account_by_session(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<Account>, AppError> {
    sqlx::query_as::<_, Account>(
        "SELECT a.id, a.email, a.username, a.display_name,
            a.password_hash, a.email_verified_at
        FROM accounts a
        JOIN account_sessions s ON s.account_id = a.id
        WHERE s.token_hash = $1 AND s.expires_at > now()
            AND a.account_status = 'active'",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn delete_session(pool: &PgPool, token_hash: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM account_sessions WHERE token_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn delete_expired_account_auth_tokens(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query("DELETE FROM email_verification_tokens WHERE expires_at <= now()")
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at <= now()")
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM account_sessions WHERE expires_at <= now()")
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM social_oauth_attempts WHERE expires_at <= now()")
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM pending_social_accounts WHERE expires_at <= now()")
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

fn map_database_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.code().as_deref() == Some("23505")
    {
        return AccountAuthError::AccountAlreadyExists.into();
    }
    AppError::Internal
}
