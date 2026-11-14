use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::account_auth::{SOCIAL_SIGNUP_LIFETIME_MINUTES, SOCIAL_STATE_LIFETIME_MINUTES},
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::account::Account,
};

pub struct OAuthAttempt {
    pub provider: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

pub struct PendingSocialAccount {
    pub provider: String,
    pub provider_subject: String,
    pub email: String,
    pub display_name: Option<String>,
}

pub async fn save_oauth_attempt(
    pool: &PgPool,
    state_hash: &str,
    provider: &str,
    nonce: &str,
    pkce_verifier: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO social_oauth_attempts (
            state_hash, provider, nonce, pkce_verifier, expires_at
        ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(state_hash)
    .bind(provider)
    .bind(nonce)
    .bind(pkce_verifier)
    .bind(Utc::now() + Duration::minutes(SOCIAL_STATE_LIFETIME_MINUTES))
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn consume_oauth_attempt(
    pool: &PgPool,
    state_hash: &str,
    provider: &str,
) -> Result<OAuthAttempt, AppError> {
    sqlx::query_as::<_, (String, String, String)>(
        "DELETE FROM social_oauth_attempts
        WHERE state_hash = $1 AND provider = $2 AND expires_at > now()
        RETURNING provider, nonce, pkce_verifier",
    )
    .bind(state_hash)
    .bind(provider)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .map(|(provider, nonce, pkce_verifier)| OAuthAttempt {
        provider,
        nonce,
        pkce_verifier,
    })
    .ok_or(AccountAuthError::SocialStateInvalid.into())
}

pub async fn find_account_by_social_identity(
    pool: &PgPool,
    provider: &str,
    provider_subject: &str,
) -> Result<Option<Account>, AppError> {
    sqlx::query_as::<_, Account>(
        "SELECT a.id, a.email, a.username, a.display_name,
            a.password_hash, a.email_verified_at
        FROM accounts a
        JOIN account_social_identities i ON i.account_id = a.id
        WHERE i.provider = $1 AND i.provider_subject = $2
            AND a.account_status = 'active'",
    )
    .bind(provider)
    .bind(provider_subject)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn save_pending_social_account(
    pool: &PgPool,
    token_hash: &str,
    identity: &PendingSocialAccount,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO pending_social_accounts (
            token_hash, provider, provider_subject,
            email, display_name, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(token_hash)
    .bind(&identity.provider)
    .bind(&identity.provider_subject)
    .bind(&identity.email)
    .bind(&identity.display_name)
    .bind(Utc::now() + Duration::minutes(SOCIAL_SIGNUP_LIFETIME_MINUTES))
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn finish_social_account_signup(
    pool: &PgPool,
    token_hash: &str,
    username: &str,
    country_code: &str,
    age_policy_version: &str,
    session_hash: &str,
    session_expires_at: DateTime<Utc>,
) -> Result<Account, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let identity = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "DELETE FROM pending_social_accounts
        WHERE token_hash = $1 AND expires_at > now()
        RETURNING provider, provider_subject, email, display_name",
    )
    .bind(token_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .map(
        |(provider, provider_subject, email, display_name)| PendingSocialAccount {
            provider,
            provider_subject,
            email,
            display_name,
        },
    )
    .ok_or(AccountAuthError::SocialSignupExpired)?;

    let account = sqlx::query_as::<_, Account>(
        "INSERT INTO accounts (
            email, username, display_name, password_hash, email_verified_at,
            eligibility_country_code, age_policy_version, terms_accepted_at
        ) VALUES ($1, $2, $3, NULL, now(), $4, $5, now())
        RETURNING id, email, username, display_name,
            password_hash, email_verified_at",
    )
    .bind(&identity.email)
    .bind(username)
    .bind(&identity.display_name)
    .bind(country_code)
    .bind(age_policy_version)
    .fetch_one(&mut *transaction)
    .await
    .map_err(map_database_error)?;

    sqlx::query(
        "INSERT INTO account_social_identities (
            account_id, provider, provider_subject
        ) VALUES ($1, $2, $3)",
    )
    .bind(account.id)
    .bind(identity.provider)
    .bind(identity.provider_subject)
    .execute(&mut *transaction)
    .await
    .map_err(map_database_error)?;

    let result = sqlx::query(
        "WITH eligible_account AS (
            SELECT id FROM accounts
            WHERE id = $1 AND account_status = 'active'
            FOR UPDATE
        )
        INSERT INTO account_sessions (account_id, token_hash, expires_at)
        SELECT eligible_account.id, $2, $3 FROM eligible_account",
    )
    .bind(account.id)
    .bind(session_hash)
    .bind(session_expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() != 1 {
        return Err(AccountAuthError::SessionInvalid.into());
    }

    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(account)
}

pub async fn create_social_session(
    pool: &PgPool,
    account_id: Uuid,
    session_hash: &str,
    session_expires_at: DateTime<Utc>,
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
    .bind(session_hash)
    .bind(session_expires_at)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() != 1 {
        return Err(AccountAuthError::SessionInvalid.into());
    }
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
