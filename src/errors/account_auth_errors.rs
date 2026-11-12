use axum::http::StatusCode;

use crate::enums::account_auth_error::AccountAuthError;

pub fn get_account_auth_error_response(error: AccountAuthError) -> (StatusCode, &'static str) {
    match error {
        AccountAuthError::InvalidCredentials => {
            (StatusCode::UNAUTHORIZED, "Invalid email or password")
        }
        AccountAuthError::SessionInvalid => (StatusCode::UNAUTHORIZED, "Sign in to continue"),
        AccountAuthError::InvalidEmail => (StatusCode::BAD_REQUEST, "Enter a valid email address"),
        AccountAuthError::CountryCodeInvalid => (
            StatusCode::BAD_REQUEST,
            "Country code must be a two-letter country code",
        ),
        AccountAuthError::PasswordLengthInvalid => (
            StatusCode::BAD_REQUEST,
            "Password must be between 12 and 128 bytes",
        ),
        AccountAuthError::UsernameInvalid => (
            StatusCode::BAD_REQUEST,
            "Username must be 3 to 24 letters, numbers, or underscores",
        ),
        AccountAuthError::TermsNotAccepted => (
            StatusCode::BAD_REQUEST,
            "Accept the terms to create an account",
        ),
        AccountAuthError::BirthDateInvalid => {
            (StatusCode::BAD_REQUEST, "Date of birth must use YYYY-MM-DD")
        }
        AccountAuthError::CountryUnavailable => (
            StatusCode::FORBIDDEN,
            "Registration is not available in this country",
        ),
        AccountAuthError::AgeRequirementNotMet => {
            (StatusCode::FORBIDDEN, "You do not meet the age requirement")
        }
        AccountAuthError::AgeCheckNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "This country requires an age-check method that is not configured",
        ),
        AccountAuthError::EmailDeliveryNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Email delivery is not configured",
        ),
        AccountAuthError::EmailDeliveryFailed => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Could not send the account email",
        ),
        AccountAuthError::EmailMustBeConfirmed => (
            StatusCode::FORBIDDEN,
            "Confirm your email before signing in",
        ),
        AccountAuthError::EmailConfirmationInvalid => (
            StatusCode::BAD_REQUEST,
            "Verification link is invalid or expired",
        ),
        AccountAuthError::PasswordResetInvalid => (
            StatusCode::BAD_REQUEST,
            "Password reset link is invalid or expired",
        ),
        AccountAuthError::SocialProviderUnsupported => (
            StatusCode::BAD_REQUEST,
            "Social sign-in provider is not supported",
        ),
        AccountAuthError::SocialProviderNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Social sign-in provider is not configured",
        ),
        AccountAuthError::SocialEmailNotVerified => (
            StatusCode::FORBIDDEN,
            "The identity provider did not confirm your email",
        ),
        AccountAuthError::SocialSignupExpired => (
            StatusCode::BAD_REQUEST,
            "Social signup has expired. Start again.",
        ),
        AccountAuthError::SocialStateInvalid => (
            StatusCode::UNAUTHORIZED,
            "Social sign-in request is invalid or expired",
        ),
        AccountAuthError::SocialProviderUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Could not connect to the identity provider",
        ),
        AccountAuthError::ExistingAccountNeedsLinking => (
            StatusCode::CONFLICT,
            "Sign in with your existing method before linking another provider",
        ),
        AccountAuthError::AccountAlreadyExists => {
            (StatusCode::CONFLICT, "Email or username is already in use")
        }
    }
}
