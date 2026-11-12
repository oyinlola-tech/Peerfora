use chrono::NaiveDate;

use crate::{
    constants::account_auth::DATE_INPUT_FORMAT, enums::account_auth_error::AccountAuthError,
};

pub fn validate_email_address(email: &str) -> Result<(), AccountAuthError> {
    email
        .parse::<lettre::Address>()
        .map(|_| ())
        .map_err(|_| AccountAuthError::InvalidEmail)
}

pub fn parse_date_of_birth(value: &str) -> Result<NaiveDate, AccountAuthError> {
    NaiveDate::parse_from_str(value, DATE_INPUT_FORMAT)
        .map_err(|_| AccountAuthError::BirthDateInvalid)
}

pub fn validate_country_code(country_code: &str) -> Result<String, AccountAuthError> {
    let normalized_code = country_code.trim().to_ascii_uppercase();
    if normalized_code.len() != 2
        || !normalized_code
            .chars()
            .all(|character| character.is_ascii_uppercase())
    {
        return Err(AccountAuthError::CountryCodeInvalid);
    }
    Ok(normalized_code)
}
