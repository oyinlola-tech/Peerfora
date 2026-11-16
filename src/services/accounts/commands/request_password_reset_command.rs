use crate::{
    errors::app_error::AppError,
    models::app_state::AppState,
    repositories::account_password_reset_repository,
    services::accounts::commands::send_account_email_command::send_password_reset_email,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
};

pub async fn request_password_reset(state: &AppState, email: &str) -> Result<(), AppError> {
    if state.email_sender.is_none() {
        return Ok(());
    }
    let token = create_one_time_token();
    let recipient = account_password_reset_repository::create_password_reset_token(
        &state.database,
        &normalize_email_address(email),
        &hash_one_time_token(&token),
    )
    .await?;
    if let Some(recipient) = recipient
        && send_password_reset_email(state, &recipient, &token)
            .await
            .is_err()
    {
        tracing::warn!("Account password reset email delivery failed");
    }
    Ok(())
}
