use crate::{
    errors::app_error::AppError,
    models::app_state::AppState,
    repositories::account_repository,
    services::accounts::commands::send_account_email_command::send_account_verification_email,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
};

pub async fn request_email_verification(state: &AppState, email: &str) -> Result<(), AppError> {
    if state.email_sender.is_none() {
        return Ok(());
    }
    let token = create_one_time_token();
    let recipient = account_repository::create_email_verification_token_for_unverified_account(
        &state.database,
        &normalize_email_address(email),
        &hash_one_time_token(&token),
    )
    .await?;
    if let Some(recipient) = recipient
        && send_account_verification_email(state, &recipient, &token)
            .await
            .is_err()
    {
        tracing::warn!("Account verification email delivery failed");
    }
    Ok(())
}
