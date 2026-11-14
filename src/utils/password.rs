use std::sync::{Arc, OnceLock};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::rngs::OsRng;
use tokio::sync::{OnceCell, Semaphore};

use crate::{enums::account_auth_error::AccountAuthError, errors::app_error::AppError};

static PASSWORD_HASHING_PERMITS: OnceLock<Arc<Semaphore>> = OnceLock::new();
static DUMMY_PASSWORD_HASH: OnceCell<String> = OnceCell::const_new();

pub async fn create_password_hash(password: String) -> Result<String, AppError> {
    let permit = acquire_password_hashing_permit().await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AppError::Internal)
    })
    .await
    .map_err(|_| AppError::Internal)?
}

pub async fn verify_password_hash(password: String, encoded_hash: String) -> Result<(), AppError> {
    let permit = acquire_password_hashing_permit().await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let parsed_hash = PasswordHash::new(&encoded_hash).map_err(|_| AppError::Internal)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::from(AccountAuthError::InvalidCredentials))
    })
    .await
    .map_err(|_| AppError::Internal)?
}

pub async fn verify_password_against_dummy_hash(password: String) -> Result<(), AppError> {
    let dummy_hash = DUMMY_PASSWORD_HASH
        .get_or_try_init(|| async {
            create_password_hash(crate::utils::one_time_token::create_one_time_token()).await
        })
        .await?
        .clone();

    match verify_password_hash(password, dummy_hash).await {
        Err(AppError::AccountAuth(AccountAuthError::InvalidCredentials)) | Ok(()) => Ok(()),
        Err(error) => Err(error),
    }
}

async fn acquire_password_hashing_permit() -> Result<tokio::sync::OwnedSemaphorePermit, AppError> {
    let semaphore = PASSWORD_HASHING_PERMITS.get_or_init(|| {
        Arc::new(Semaphore::new(
            crate::constants::account_auth::PASSWORD_HASHING_MAX_CONCURRENCY,
        ))
    });
    Arc::clone(semaphore)
        .acquire_owned()
        .await
        .map_err(|_| AppError::Internal)
}
