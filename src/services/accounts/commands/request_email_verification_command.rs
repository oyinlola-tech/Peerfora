use sqlx::PgPool;

use crate::{
    errors::app_error::AppError,
    repositories::account_repository,
    utils::{
        email_address::normalize_email_address,
        one_time_token::{create_one_time_token, hash_one_time_token},
    },
};

pub async fn request_email_verification(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(String, String)>, AppError> {
    let token = create_one_time_token();
    let email = account_repository::create_email_verification_token_for_unverified_account(
        pool,
        &normalize_email_address(email),
        &hash_one_time_token(&token),
    )
    .await?;
    Ok(email.map(|email| (email, token)))
}
