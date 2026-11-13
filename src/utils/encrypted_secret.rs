use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, consts::U12},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::{RngCore, rngs::OsRng};

use crate::{enums::admin_auth_error::AdminAuthError, errors::app_error::AppError};

const NONCE_BYTES: usize = 12;

pub fn encrypt_secret(secret: &[u8], encoded_key: &str) -> Result<String, AppError> {
    let key = decode_encryption_key(encoded_key)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::Internal)?;
    let mut nonce_bytes = vec![0_u8; NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::<U12>::try_from(nonce_bytes.as_slice()).map_err(|_| AppError::Internal)?;
    let encrypted = cipher
        .encrypt(&nonce, secret)
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
    let nonce = Nonce::<U12>::try_from(nonce_bytes).map_err(|_| AppError::Internal)?;
    cipher
        .decrypt(&nonce, encrypted)
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

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};

    use super::{decrypt_secret, encrypt_secret};

    #[test]
    fn encrypted_secret_can_only_be_opened_with_its_key() {
        let key = STANDARD.encode([11_u8; 32]);
        let other_key = STANDARD.encode([12_u8; 32]);
        let ciphertext = encrypt_secret(b"authenticator-secret", &key).expect("encrypt secret");

        assert_eq!(
            decrypt_secret(&ciphertext, &key).expect("decrypt with correct key"),
            b"authenticator-secret"
        );
        assert!(decrypt_secret(&ciphertext, &other_key).is_err());
    }
}
