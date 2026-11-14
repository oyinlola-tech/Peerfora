use crate::constants::account_auth::{
    EMAIL_VERIFICATION_LIFETIME_HOURS, EMAIL_VERIFICATION_SUBJECT, PASSWORD_RESET_LIFETIME_HOURS,
    PASSWORD_RESET_SUBJECT,
};

use super::EmailMessage;

pub fn build_email_verification_message(verification_link: &str) -> EmailMessage {
    EmailMessage {
        subject: EMAIL_VERIFICATION_SUBJECT,
        plain_text: format!(
            "Hello,\n\nPlease confirm your email address to finish creating your Peerfora account. Open the secure, one-time link below:\n{verification_link}\n\nThis link expires in {EMAIL_VERIFICATION_LIFETIME_HOURS} hours. After you confirm, return to Peerfora and sign in with your email and password.\n\nIf you did not start creating a Peerfora account, you can ignore this message. Do not forward this link or share it with anyone.\n\nPeerfora"
        ),
    }
}

pub fn build_password_reset_message(reset_link: &str) -> EmailMessage {
    EmailMessage {
        subject: PASSWORD_RESET_SUBJECT,
        plain_text: format!(
            "Hello,\n\nWe received a request to reset the password for your Peerfora account. To choose a new password, open the secure, one-time link below:\n{reset_link}\n\nThis link expires in {PASSWORD_RESET_LIFETIME_HOURS} hour. After a successful password change, your existing sessions are revoked and you will need to sign in again.\n\nIf you did not request a password reset, ignore this message and do not share or forward the link. Your password will not change unless the link is used.\n\nPeerfora"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_email_verification_message, build_password_reset_message};
    use crate::constants::{
        account_auth::ONE_TIME_TOKEN_QUERY_PARAMETER, runtime_settings::DEFAULT_PUBLIC_URL,
    };

    const TEST_EMAIL_VERIFICATION_PATH: &str = "/verify";
    const TEST_PASSWORD_RESET_PATH: &str = "/reset";
    const TEST_ONE_TIME_TOKEN: &str = "x";

    #[test]
    fn email_verification_message_explains_the_link_and_unexpected_email_safety() {
        let verification_link = format!(
            "{DEFAULT_PUBLIC_URL}{TEST_EMAIL_VERIFICATION_PATH}?{ONE_TIME_TOKEN_QUERY_PARAMETER}={TEST_ONE_TIME_TOKEN}"
        );
        let message = build_email_verification_message(&verification_link);

        assert!(message.plain_text.contains(&verification_link));
        assert!(message.plain_text.contains("24 hours"));
        assert!(message.plain_text.contains("ignore this message"));
    }

    #[test]
    fn password_reset_message_explains_expiry_and_session_revocation() {
        let reset_link = format!(
            "{DEFAULT_PUBLIC_URL}{TEST_PASSWORD_RESET_PATH}?{ONE_TIME_TOKEN_QUERY_PARAMETER}={TEST_ONE_TIME_TOKEN}"
        );
        let message = build_password_reset_message(&reset_link);

        assert!(message.plain_text.contains(&reset_link));
        assert!(message.plain_text.contains("1 hour"));
        assert!(message.plain_text.contains("existing sessions are revoked"));
        assert!(message.plain_text.contains("Your password will not change"));
    }
}
