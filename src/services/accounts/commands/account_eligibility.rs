use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Utc};

use crate::{
    configs::settings::CountryPolicy,
    constants::account_auth::{GLOBAL_AGE_POLICY_VERSION, MINIMUM_ACCOUNT_AGE_YEARS},
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
};

pub fn require_account_age_eligibility(
    policies: &HashMap<String, CountryPolicy>,
    country_code: &str,
    date_of_birth: NaiveDate,
) -> Result<CountryPolicy, AppError> {
    require_account_age_eligibility_on_date(
        policies,
        country_code,
        date_of_birth,
        Utc::now().date_naive(),
    )
}

fn require_account_age_eligibility_on_date(
    policies: &HashMap<String, CountryPolicy>,
    country_code: &str,
    date_of_birth: NaiveDate,
    today: NaiveDate,
) -> Result<CountryPolicy, AppError> {
    let policy = policies
        .get(country_code)
        .cloned()
        .unwrap_or(CountryPolicy {
            minimum_age: MINIMUM_ACCOUNT_AGE_YEARS,
            policy_version: GLOBAL_AGE_POLICY_VERSION.to_owned(),
            age_assurance_required: false,
        });
    if policy.age_assurance_required {
        return Err(AccountAuthError::AgeCheckNotConfigured.into());
    }
    if !meets_minimum_age(date_of_birth, today, policy.minimum_age) {
        return Err(AccountAuthError::AgeRequirementNotMet.into());
    }
    Ok(policy)
}

fn meets_minimum_age(date_of_birth: NaiveDate, today: NaiveDate, country_minimum_age: u8) -> bool {
    let mut age = today.year() - date_of_birth.year();
    if (today.month(), today.day()) < (date_of_birth.month(), date_of_birth.day()) {
        age -= 1;
    }
    age >= i32::from(country_minimum_age.max(MINIMUM_ACCOUNT_AGE_YEARS))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;

    use super::{meets_minimum_age, require_account_age_eligibility_on_date};
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
        let today = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
        let seventeen_years_ago = NaiveDate::from_ymd_opt(2009, 11, 13).unwrap();

        let result =
            require_account_age_eligibility_on_date(&policies, "NG", seventeen_years_ago, today);

        assert!(matches!(
            result,
            Err(AppError::AccountAuth(
                AccountAuthError::AgeRequirementNotMet
            ))
        ));
    }

    #[test]
    fn adult_registration_is_allowed_when_country_policy_is_configured() {
        let policies = HashMap::new();
        let today = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
        let eighteen_years_ago = NaiveDate::from_ymd_opt(2008, 11, 13).unwrap();

        let policy =
            require_account_age_eligibility_on_date(&policies, "CA", eighteen_years_ago, today)
                .expect("the default global policy should allow an eligible adult");

        assert_eq!(policy.policy_version, "peerfora-global-18-plus-v1");
    }

    #[test]
    fn age_eligibility_does_not_depend_on_education_level() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
        let adult_high_school_student_birth_date = NaiveDate::from_ymd_opt(2008, 11, 13).unwrap();

        assert!(meets_minimum_age(
            adult_high_school_student_birth_date,
            today,
            18
        ));
    }

    #[test]
    fn birthday_has_not_passed_when_current_calendar_day_is_before_birthday() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
        let next_day_before_birthday = NaiveDate::from_ymd_opt(2008, 11, 14).unwrap();

        assert!(!meets_minimum_age(next_day_before_birthday, today, 18));
    }

    #[test]
    fn future_birth_date_is_not_eligible() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
        let future_birth_date = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();

        assert!(!meets_minimum_age(future_birth_date, today, 18));
    }

    #[test]
    fn leap_day_birthday_uses_calendar_age() {
        let birthday = NaiveDate::from_ymd_opt(2004, 2, 29).unwrap();
        let before_birthday = NaiveDate::from_ymd_opt(2022, 2, 28).unwrap();
        let on_birthday = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();

        assert!(!meets_minimum_age(birthday, before_birthday, 18));
        assert!(meets_minimum_age(birthday, on_birthday, 20));
    }
}
