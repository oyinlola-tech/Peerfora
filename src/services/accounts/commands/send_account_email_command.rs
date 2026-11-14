use crate::{
    constants::account_auth::{
        ONE_TIME_TOKEN_QUERY_PARAMETER, PASSWORD_RESET_PAGE_PATH, VERIFY_EMAIL_ROUTE,
    },
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    integrations::email::templates::account_access_email_templates::{
        build_email_verification_message, build_password_reset_message,
    },
    models::app_state::AppState,
};

pub async fn send_account_verification_email(
    state: &AppState,
    recipient: &str,
    token: &str,
) -> Result<(), AppError> {
    let verification_link = format!(
        "{}{path}?{ONE_TIME_TOKEN_QUERY_PARAMETER}={token}",
        state.public_url.trim_end_matches('/'),
        path = VERIFY_EMAIL_ROUTE
    );
    let message = build_email_verification_message(&verification_link);
    send_email(state, recipient, message.subject, &message.plain_text).await
}

pub async fn send_password_reset_email(
    state: &AppState,
    recipient: &str,
    token: &str,
) -> Result<(), AppError> {
    let reset_link = format!(
        "{}{path}?{ONE_TIME_TOKEN_QUERY_PARAMETER}={token}",
        state.public_url.trim_end_matches('/'),
        path = PASSWORD_RESET_PAGE_PATH
    );
    let message = build_password_reset_message(&reset_link);
    send_email(state, recipient, message.subject, &message.plain_text).await
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
