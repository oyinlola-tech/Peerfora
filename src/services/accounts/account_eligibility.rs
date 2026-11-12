use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Utc};

use crate::{
    configs::settings::CountryPolicy, enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
};

pub fn require_account_age_eligibility(
    policies: &HashMap<String, CountryPolicy>,
    country_code: &str,
    date_of_birth: NaiveDate,
) -> Result<&CountryPolicy, AppError> {
    let policy = policies
        .get(country_code)
        .ok_or(AccountAuthError::CountryUnavailable)?;
    if policy.age_assurance_required {
        return Err(AccountAuthError::AgeCheckNotConfigured.into());
    }
    if calculate_age(date_of_birth) < i32::from(policy.minimum_age) {
        return Err(AccountAuthError::AgeRequirementNotMet.into());
    }
    Ok(policy)
}

fn calculate_age(date_of_birth: NaiveDate) -> i32 {
    let today = Utc::now().date_naive();
    let mut age = today.year() - date_of_birth.year();
    if (today.month(), today.day()) < (date_of_birth.month(), date_of_birth.day()) {
        age -= 1;
    }
    age
}
