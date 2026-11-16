use crate::{
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    integrations::email::{
        links::{EmailLink, build_email_link},
        templates::transactional_email::{TransactionalEmail, render_email},
    },
    interfaces::email_sender::EmailMessage,
    models::app_state::AppState,
};

pub async fn send_account_verification_email(
    state: &AppState,
    recipient: &str,
    token: &str,
) -> Result<(), AppError> {
    let link = build_email_link(&state.public_url, EmailLink::VerifyEmail, token)?;
    let message = render_email(TransactionalEmail::VerifyEmail { link: &link }, None)?;
    send_email(state, recipient, &message).await
}

pub async fn send_password_reset_email(
    state: &AppState,
    recipient: &str,
    token: &str,
) -> Result<(), AppError> {
    let link = build_email_link(&state.public_url, EmailLink::ResetPassword, token)?;
    let message = render_email(TransactionalEmail::ResetPassword { link: &link }, None)?;
    send_email(state, recipient, &message).await
}

async fn send_email(
    state: &AppState,
    recipient: &str,
    message: &EmailMessage,
) -> Result<(), AppError> {
    state
        .email_sender
        .as_ref()
        .ok_or(AccountAuthError::EmailDeliveryNotConfigured)?
        .send_email(recipient, message)
        .await
        .map_err(AppError::from)
}
