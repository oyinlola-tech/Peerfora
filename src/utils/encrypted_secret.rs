use aes_gcm::{Aes256Gcm, Nonce, aead::{Aead, KeyInit}};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::{RngCore, rngs::OsRng};

use crate::{enums::admin_auth_error::AdminAuthError, errors::app_error::AppError};

const NONCE_BYTES: usize = 12;

pub fn encrypt_secret(secret: &[u8], encoded_key: &str) -> Result<String, AppError> {
    let key = decode_encryption_key(encoded_key)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::Internal)?;
    let mut nonce_bytes = vec![0_u8; NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce_bytes);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), secret)
        .map_err(|_| AppError::Internal)?;
    nonce_bytes.extend_from_slice(&encrypted);
    Ok(STANDARD.encode(nonce_bytes))
}

pub fn decrypt_secret(encrypted_secret: &str, encoded_key: &str) -> Result<Vec<u8>, AppError> {
    let key = decode_encryption_key(encoded_key)?;
    let ciphertext = STANDARD
        .decode(encrypted_secret)
        .map_err(|_| AppError::Internal)?;
    if ciphertext.len() <= NONCE_BYTES {
        return Err(AppError::Internal);
    }
    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_BYTES);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::Internal)?;
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), encrypted)
        .map_err(|_| AppError::Internal)
}

fn decode_encryption_key(encoded_key: &str) -> Result<Vec<u8>, AppError> {
    let key = STANDARD
        .decode(encoded_key)
        .map_err(|_| AdminAuthError::MfaEncryptionKeyUnavailable)?;
    if key.len() != 32 {
        return Err(AdminAuthError::MfaEncryptionKeyUnavailable.into());
    }
    Ok(key)
}
