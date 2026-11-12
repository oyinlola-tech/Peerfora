use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::rngs::OsRng;

use crate::{enums::account_auth_error::AccountAuthError, errors::app_error::AppError};

pub async fn create_password_hash(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
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
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&encoded_hash).map_err(|_| AppError::Internal)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::from(AccountAuthError::InvalidCredentials))
    })
    .await
    .map_err(|_| AppError::Internal)?
}
