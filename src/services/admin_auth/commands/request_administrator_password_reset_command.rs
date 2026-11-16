use crate::{
    errors::app_error::AppError,
    integrations::email::administrator_password_reset_email::send_administrator_password_reset_email,
    models::app_state::AppState,
    repositories::admin_auth_repository,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
};

pub async fn request_administrator_password_reset(
    state: &AppState,
    email: &str,
) -> Result<(), AppError> {
    let Some(email_sender) = state.email_sender.as_deref() else {
        return Ok(());
    };
    let token = create_one_time_token();
    let token_hash = hash_one_time_token(&token);
    let recipient = admin_auth_repository::create_administrator_password_reset_token(
        &state.database,
        &normalize_email_address(email),
        &token_hash,
    )
    .await?;
    if let Some(recipient) = recipient
        && send_administrator_password_reset_email(
            email_sender,
            &recipient.email,
            &token,
            state.public_url.trim_end_matches('/'),
        )
        .await
        .is_err()
    {
        tracing::warn!("administrator password reset email delivery failed");
        if admin_auth_repository::revoke_failed_administrator_password_reset_delivery(
            &state.database,
            recipient.administrator_id,
            &token_hash,
        )
        .await
        .is_err()
        {
            tracing::warn!("administrator password reset token cleanup failed");
        }
    }
    Ok(())
}
