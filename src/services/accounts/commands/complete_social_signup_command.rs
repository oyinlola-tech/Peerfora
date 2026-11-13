use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;

use crate::{
    configs::settings::CountryPolicy,
    constants::account_auth::{MAX_USERNAME_LENGTH, MIN_USERNAME_LENGTH, SESSION_LIFETIME_DAYS},
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::account::Account,
    repositories::social_auth_repository,
    services::accounts::commands::account_eligibility::require_account_age_eligibility,
    utils::one_time_token::{create_one_time_token, hash_one_time_token},
};

pub struct SocialSignupDetails {
    pub onboarding_token: String,
    pub username: String,
    pub date_of_birth: NaiveDate,
    pub country_code: String,
    pub accepted_terms: bool,
}

pub async fn complete_social_signup(
    pool: &PgPool,
    country_policies: &std::collections::HashMap<String, CountryPolicy>,
    details: SocialSignupDetails,
) -> Result<(Account, String), AppError> {
    validate_social_signup(&details)?;
    let country_code = details.country_code.trim().to_ascii_uppercase();
    let eligibility_policy =
        require_account_age_eligibility(country_policies, &country_code, details.date_of_birth)?;
    let session_token = create_one_time_token();
    let expires_at = Utc::now() + Duration::days(SESSION_LIFETIME_DAYS);
    let account = social_auth_repository::finish_social_account_signup(
        pool,
        &hash_one_time_token(&details.onboarding_token),
        &details.username.trim().to_ascii_lowercase(),
        &country_code,
        &eligibility_policy.policy_version,
        &hash_one_time_token(&session_token),
        expires_at,
    )
    .await?;
    Ok((account, session_token))
}

fn validate_social_signup(details: &SocialSignupDetails) -> Result<(), AppError> {
    let username = details.username.trim();
    let valid_length = (MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&username.len());
    let valid_characters = username
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_');

    if !valid_length || !valid_characters {
        return Err(AccountAuthError::UsernameInvalid.into());
    }
    if !details.accepted_terms {
        return Err(AccountAuthError::TermsNotAccepted.into());
    }
    Ok(())
}
