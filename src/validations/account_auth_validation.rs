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

pub fn validate_verification_token(token: &str) -> Result<(), AccountAuthError> {
    if token.len() != crate::constants::one_time_token::ONE_TIME_TOKEN_RANDOM_BYTE_LENGTH * 2
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(AccountAuthError::EmailConfirmationInvalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_verification_token;

    #[test]
    fn verification_form_only_accepts_lowercase_hex_tokens() {
        assert!(validate_verification_token(&"ab012345".repeat(8)).is_ok());
        assert!(validate_verification_token(&"A".repeat(64)).is_err());
        assert!(validate_verification_token(&"a".repeat(63)).is_err());
        assert!(validate_verification_token(&format!("{}\"", "a".repeat(63))).is_err());
    }
}
