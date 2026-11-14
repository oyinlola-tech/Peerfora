use sqlx::PgPool;

use crate::{
    errors::app_error::AppError,
    models::admin_identity::AdministratorPasswordResetRecipient,
    repositories::admin_auth_repository,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
};

pub struct AdministratorPasswordResetRequest {
    pub recipient: Option<AdministratorPasswordResetRecipient>,
    pub token: String,
    pub token_hash: String,
}

pub async fn request_administrator_password_reset(
    pool: &PgPool,
    email: &str,
) -> Result<AdministratorPasswordResetRequest, AppError> {
    let token = create_one_time_token();
    let token_hash = hash_one_time_token(&token);
    let recipient = admin_auth_repository::create_administrator_password_reset_token(
        pool,
        &normalize_email_address(email),
        &token_hash,
    )
    .await?;
    Ok(AdministratorPasswordResetRequest {
        recipient,
        token,
        token_hash,
    })
}
