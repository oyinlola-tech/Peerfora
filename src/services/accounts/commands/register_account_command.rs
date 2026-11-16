use chrono::NaiveDate;
use sqlx::PgPool;

use crate::{
    configs::settings::CountryPolicy,
    constants::account_auth::{
        MAX_EMAIL_LENGTH, MAX_PASSWORD_LENGTH_BYTES, MAX_USERNAME_LENGTH,
        MIN_PASSWORD_LENGTH_BYTES, MIN_USERNAME_LENGTH,
    },
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::account::Account,
    repositories::account_repository,
    services::academic_directory::queries::find_active_country_query,
    services::accounts::commands::account_eligibility::require_account_age_eligibility,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
        password::create_password_hash,
    },
};

pub struct NewAccount {
    pub email: String,
    pub password: String,
    pub username: String,
    pub display_name: Option<String>,
    pub date_of_birth: NaiveDate,
    pub country_code: String,
    pub accepted_terms: bool,
}

pub async fn register_account(
    pool: &PgPool,
    policies: &std::collections::HashMap<String, CountryPolicy>,
    input: NewAccount,
) -> Result<(Account, String), AppError> {
    validate_new_account(&input)?;
    if !find_active_country_query::is_country_code_supported(pool, &input.country_code).await? {
        return Err(AccountAuthError::CountryCodeUnknown.into());
    }
    let policy =
        require_account_age_eligibility(policies, &input.country_code, input.date_of_birth)?;
    let password_hash = create_password_hash(input.password).await?;
    let verification_token = create_one_time_token();
    let email = normalize_email_address(&input.email);
    let username = input.username.trim().to_ascii_lowercase();
    let verification_token_hash = hash_one_time_token(&verification_token);
    let account = account_repository::create_account(
        pool,
        account_repository::NewAccountRecord {
            email: &email,
            username: &username,
            display_name: input.display_name.as_deref(),
            password_hash: &password_hash,
            country_code: &input.country_code,
            age_policy_version: &policy.policy_version,
            verification_token_hash: &verification_token_hash,
        },
    )
    .await?;
    Ok((account, verification_token))
}

fn validate_new_account(input: &NewAccount) -> Result<(), AppError> {
    if input.email.len() > MAX_EMAIL_LENGTH {
        return Err(AccountAuthError::InvalidEmail.into());
    }
    if !(MIN_PASSWORD_LENGTH_BYTES..=MAX_PASSWORD_LENGTH_BYTES).contains(&input.password.len()) {
        return Err(AccountAuthError::PasswordLengthInvalid.into());
    }
    let username = input.username.trim();
    if !(MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&username.len())
        || !username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(AccountAuthError::UsernameInvalid.into());
    }
    if !input.accepted_terms {
        return Err(AccountAuthError::TermsNotAccepted.into());
    }
    Ok(())
}
