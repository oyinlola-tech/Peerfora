use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::{
    constants::account_auth::SESSION_LIFETIME_DAYS,
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::account::Account,
    repositories::account_repository,
    utils::{
        one_time_token::{create_one_time_token, hash_one_time_token},
        password::verify_password_hash,
    },
};

pub struct AccountSession {
    pub account: Account,
    pub token: String,
}

pub async fn login_account(
    pool: &PgPool,
    email: &str,
    password: String,
) -> Result<AccountSession, AppError> {
    let account =
        account_repository::find_account_by_email(pool, &email.trim().to_ascii_lowercase())
            .await?
            .ok_or(AccountAuthError::InvalidCredentials)?;
    let password_hash = account
        .password_hash
        .clone()
        .ok_or(AccountAuthError::InvalidCredentials)?;
    verify_password_hash(password, password_hash).await?;
    if account.email_verified_at.is_none() {
        return Err(AccountAuthError::EmailMustBeConfirmed.into());
    }
    let token = create_one_time_token();
    let expires_at = Utc::now() + Duration::days(SESSION_LIFETIME_DAYS);
    account_repository::create_session(pool, account.id, &hash_one_time_token(&token), expires_at)
        .await?;
    Ok(AccountSession { account, token })
}
