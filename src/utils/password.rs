use std::sync::{Arc, OnceLock};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::rngs::OsRng;
use tokio::sync::Semaphore;

use crate::{enums::account_auth_error::AccountAuthError, errors::app_error::AppError};

static PASSWORD_HASHING_PERMITS: OnceLock<Arc<Semaphore>> = OnceLock::new();

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
