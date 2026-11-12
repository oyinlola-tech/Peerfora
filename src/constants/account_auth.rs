pub const SESSION_COOKIE_NAME: &str = "peerfora_session";
pub const SOCIAL_STATE_COOKIE_NAME: &str = "peerfora_oauth_state";
pub const SOCIAL_SIGNUP_COOKIE_NAME: &str = "peerfora_social_signup";

pub const SESSION_LIFETIME_SECONDS: u32 = 2_592_000;
pub const SOCIAL_STATE_LIFETIME_SECONDS: u32 = 600;
pub const SOCIAL_SIGNUP_LIFETIME_SECONDS: u32 = 1_800;
pub const SESSION_LIFETIME_DAYS: i64 = 30;
pub const SOCIAL_STATE_LIFETIME_MINUTES: i64 = 10;
pub const SOCIAL_SIGNUP_LIFETIME_MINUTES: i64 = 30;
pub const EMAIL_VERIFICATION_LIFETIME_HOURS: i64 = 24;
pub const PASSWORD_RESET_LIFETIME_HOURS: i64 = 1;
pub const EXPIRED_AUTH_TOKEN_CLEANUP_INTERVAL_SECONDS: u64 = 3_600;

pub const MIN_PASSWORD_LENGTH_BYTES: usize = 12;
pub const MAX_PASSWORD_LENGTH_BYTES: usize = 128;
pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MAX_USERNAME_LENGTH: usize = 24;
pub const MAX_EMAIL_LENGTH: usize = 254;
pub const DATE_INPUT_FORMAT: &str = "%Y-%m-%d";

pub const REGISTER_ACCOUNT_ROUTE: &str = "/api/v1/auth/register";
pub const LOGIN_ACCOUNT_ROUTE: &str = "/api/v1/auth/login";
pub const VERIFY_EMAIL_ROUTE: &str = "/api/v1/auth/verify-email";
pub const REQUEST_PASSWORD_RESET_ROUTE: &str = "/api/v1/auth/password-reset/request";
pub const CONFIRM_PASSWORD_RESET_ROUTE: &str = "/api/v1/auth/password-reset/confirm";
pub const CURRENT_ACCOUNT_ROUTE: &str = "/api/v1/auth/me";
pub const LOGOUT_ACCOUNT_ROUTE: &str = "/api/v1/auth/logout";
pub const SOCIAL_SIGNIN_START_ROUTE: &str = "/api/v1/auth/social/{provider}/start";
pub const SOCIAL_SIGNIN_CALLBACK_ROUTE: &str = "/api/v1/auth/social/{provider}/callback";
pub const COMPLETE_SOCIAL_SIGNUP_ROUTE: &str = "/api/v1/auth/social/complete";

pub const SOCIAL_SIGNUP_PAGE_PATH: &str = "/complete-social-signup";
pub const PASSWORD_RESET_PAGE_PATH: &str = "/reset-password";
pub const SESSION_COOKIE_PATH: &str = "/";
pub const SOCIAL_AUTH_COOKIE_PATH: &str = "/api/v1/auth/social";
pub const SOCIAL_SIGNUP_COOKIE_PATH: &str = COMPLETE_SOCIAL_SIGNUP_ROUTE;
pub const EMAIL_VERIFICATION_SUBJECT: &str = "Confirm your Peerfora email";
pub const PASSWORD_RESET_SUBJECT: &str = "Reset your Peerfora password";
pub const EMAIL_VERIFICATION_MESSAGE: &str = "Open this link to confirm your email";
pub const PASSWORD_RESET_MESSAGE: &str = "Open this link to reset your password";
pub const REGISTRATION_SUBMITTED_MESSAGE: &str = "Check your email to confirm your account";
pub const EMAIL_CONFIRMED_MESSAGE: &str = "Email confirmed. You can now sign in.";
pub const PASSWORD_RESET_REQUESTED_MESSAGE: &str =
    "If the account exists, password reset instructions have been sent";
pub const PASSWORD_RESET_COMPLETED_MESSAGE: &str =
    "Password updated. Sign in again with your new password.";
