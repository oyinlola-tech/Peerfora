use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Utc};

use crate::{
    configs::settings::CountryPolicy, constants::account_auth::MINIMUM_ACCOUNT_AGE_YEARS,
    enums::account_auth_error::AccountAuthError, errors::app_error::AppError,
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
    let minimum_age = policy.minimum_age.max(MINIMUM_ACCOUNT_AGE_YEARS);
    if calculate_age(date_of_birth) < i32::from(minimum_age) {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{Duration, Utc};

    use super::require_account_age_eligibility;
    use crate::{
        configs::settings::CountryPolicy, enums::account_auth_error::AccountAuthError,
        errors::app_error::AppError,
    };

    fn country_policy(minimum_age: u8) -> CountryPolicy {
        CountryPolicy {
            minimum_age,
            policy_version: "test-policy".to_owned(),
            age_assurance_required: false,
        }
    }

    #[test]
    fn configured_policy_cannot_allow_registration_below_eighteen() {
        let policies = HashMap::from([("NG".to_owned(), country_policy(16))]);
        let seventeen_years_ago = Utc::now().date_naive() - Duration::days(365 * 17);

        let result = require_account_age_eligibility(&policies, "NG", seventeen_years_ago);

        assert!(matches!(
            result,
            Err(AppError::AccountAuth(
                AccountAuthError::AgeRequirementNotMet
            ))
        ));
    }

    #[test]
    fn adult_registration_is_allowed_when_country_policy_is_configured() {
        let policies = HashMap::from([("NG".to_owned(), country_policy(18))]);
        let nineteen_years_ago = Utc::now().date_naive() - Duration::days(365 * 19);

        assert!(require_account_age_eligibility(&policies, "NG", nineteen_years_ago).is_ok());
    }
}
