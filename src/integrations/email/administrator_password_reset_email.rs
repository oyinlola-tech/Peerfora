use crate::{
    errors::app_error::AppError,
    integrations::email::{
        links::{EmailLink, build_email_link},
        templates::transactional_email::{TransactionalEmail, render_email},
    },
    interfaces::email_sender::EmailSender,
};

pub async fn send_administrator_password_reset_email(
    email_sender: &dyn EmailSender,
    recipient_email: &str,
    reset_token: &str,
    public_url: &str,
) -> Result<(), AppError> {
    let link = build_email_link(public_url, EmailLink::AdministratorReset, reset_token)?;
    let message = render_email(TransactionalEmail::AdministratorReset { link: &link }, None)?;
    email_sender
        .send_email(recipient_email, &message)
        .await
        .map_err(AppError::from)
}
