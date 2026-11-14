use crate::{
    enums::account_auth_error::AccountAuthError,
    integrations::email::templates::administrator_password_reset_email_template::build_administrator_password_reset_email,
    interfaces::email_sender::EmailSender,
};

pub async fn send_administrator_password_reset_email(
    email_sender: &dyn EmailSender,
    recipient_email: &str,
    reset_token: &str,
    public_url: &str,
) -> Result<(), AccountAuthError> {
    let message = build_administrator_password_reset_email(reset_token, public_url);
    email_sender
        .send_email(recipient_email, message.subject, &message.plain_text)
        .await
}
