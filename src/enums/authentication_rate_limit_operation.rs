use crate::constants::authentication_rate_limits::{
    ACCOUNT_LOGIN_MAXIMUM_REQUESTS, ACCOUNT_LOGIN_WINDOW_SECONDS,
    ACCOUNT_PASSWORD_RESET_MAXIMUM_REQUESTS, ACCOUNT_PASSWORD_RESET_WINDOW_SECONDS,
    ACCOUNT_REGISTRATION_MAXIMUM_REQUESTS, ACCOUNT_REGISTRATION_WINDOW_SECONDS,
    ADMINISTRATOR_LOGIN_MAXIMUM_REQUESTS, ADMINISTRATOR_LOGIN_WINDOW_SECONDS,
    ADMINISTRATOR_PASSWORD_RESET_MAXIMUM_REQUESTS, ADMINISTRATOR_PASSWORD_RESET_WINDOW_SECONDS,
};

#[derive(Clone, Copy)]
pub enum AuthenticationRateLimitOperation {
    AccountRegistration,
    AccountLogin,
    AccountPasswordReset,
    AdministratorLogin,
    AdministratorPasswordReset,
}

impl AuthenticationRateLimitOperation {
    pub fn database_key(self) -> &'static str {
        match self {
            Self::AccountRegistration => "account_registration",
            Self::AccountLogin => "account_login",
            Self::AccountPasswordReset => "account_password_reset",
            Self::AdministratorLogin => "administrator_login",
            Self::AdministratorPasswordReset => "administrator_password_reset",
        }
    }

    pub fn maximum_requests(self) -> i32 {
        match self {
            Self::AccountRegistration => ACCOUNT_REGISTRATION_MAXIMUM_REQUESTS,
            Self::AccountLogin => ACCOUNT_LOGIN_MAXIMUM_REQUESTS,
            Self::AccountPasswordReset => ACCOUNT_PASSWORD_RESET_MAXIMUM_REQUESTS,
            Self::AdministratorLogin => ADMINISTRATOR_LOGIN_MAXIMUM_REQUESTS,
            Self::AdministratorPasswordReset => ADMINISTRATOR_PASSWORD_RESET_MAXIMUM_REQUESTS,
        }
    }

    pub fn window_seconds(self) -> i32 {
        match self {
            Self::AccountLogin => ACCOUNT_LOGIN_WINDOW_SECONDS,
            Self::AdministratorLogin => ADMINISTRATOR_LOGIN_WINDOW_SECONDS,
            Self::AccountRegistration => ACCOUNT_REGISTRATION_WINDOW_SECONDS,
            Self::AccountPasswordReset => ACCOUNT_PASSWORD_RESET_WINDOW_SECONDS,
            Self::AdministratorPasswordReset => ADMINISTRATOR_PASSWORD_RESET_WINDOW_SECONDS,
        }
    }
}
