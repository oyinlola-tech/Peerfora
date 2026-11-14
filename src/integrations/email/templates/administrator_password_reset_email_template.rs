use crate::constants::{
    account_auth::ONE_TIME_TOKEN_QUERY_PARAMETER,
    admin_auth::{
        ADMIN_PASSWORD_RESET_EMAIL_SUBJECT, ADMIN_PASSWORD_RESET_LIFETIME_MINUTES,
        ADMIN_PASSWORD_RESET_PAGE_PATH,
    },
};

use super::EmailMessage;

pub fn build_administrator_password_reset_email(
    reset_token: &str,
    public_url: &str,
) -> EmailMessage {
    let reset_link = build_administrator_password_reset_link(reset_token, public_url);
    EmailMessage {
        subject: ADMIN_PASSWORD_RESET_EMAIL_SUBJECT,
        plain_text: format_administrator_password_reset_message(&reset_link),
    }
}

fn build_administrator_password_reset_link(reset_token: &str, public_url: &str) -> String {
    format!(
        "{}{ADMIN_PASSWORD_RESET_PAGE_PATH}?{ONE_TIME_TOKEN_QUERY_PARAMETER}={reset_token}",
        public_url.trim_end_matches('/')
    )
}

fn format_administrator_password_reset_message(reset_link: &str) -> String {
    format!(
        "Hello,\n\nWe received a request to reset the password for your Peerfora administrator account. To choose a new password, open this secure, one-time link:\n{reset_link}\n\nThe link expires in {ADMIN_PASSWORD_RESET_LIFETIME_MINUTES} minutes. After a successful reset, all active administrator sessions and pending multifactor login challenges are revoked. You must sign in again and complete multifactor authentication. This reset does not replace or disable your authenticator.\n\nIf you did not request a reset, ignore this email. Your password will not change unless the link is used. Do not forward this link or share it with anyone.\n\nPeerfora Administration"
    )
}
