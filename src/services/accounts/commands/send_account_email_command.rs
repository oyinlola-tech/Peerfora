use crate::{
    constants::account_auth::{
        EMAIL_VERIFICATION_MESSAGE, EMAIL_VERIFICATION_SUBJECT, PASSWORD_RESET_MESSAGE,
        PASSWORD_RESET_PAGE_PATH, PASSWORD_RESET_SUBJECT, VERIFY_EMAIL_ROUTE,
    },
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::app_state::AppState,
};

pub async fn send_account_verification_email(
    state: &AppState,
    recipient: &str,
    token: &str,
) -> Result<(), AppError> {
    let verification_link = format!(
        "{}{path}?token={token}",
        state.public_url.trim_end_matches('/'),
        path = VERIFY_EMAIL_ROUTE
    );
    send_email(
        state,
        recipient,
        EMAIL_VERIFICATION_SUBJECT,
        &format!("{EMAIL_VERIFICATION_MESSAGE}: {verification_link}"),
    )
    .await
}

pub async fn send_password_reset_email(
    state: &AppState,
    recipient: &str,
    token: &str,
) -> Result<(), AppError> {
    let reset_link = format!(
        "{}{path}?token={token}",
        state.public_url.trim_end_matches('/'),
        path = PASSWORD_RESET_PAGE_PATH
    );
    send_email(
        state,
        recipient,
        PASSWORD_RESET_SUBJECT,
        &format!("{PASSWORD_RESET_MESSAGE}: {reset_link}"),
    )
    .await
}

async fn send_email(
    state: &AppState,
    recipient: &str,
    subject: &str,
    message: &str,
) -> Result<(), AppError> {
    let email_sender = state
        .email_sender
        .as_ref()
        .ok_or(AccountAuthError::EmailDeliveryNotConfigured)?;
    email_sender
        .send_email(recipient, subject, message)
        .await
        .map_err(AppError::from)
}
