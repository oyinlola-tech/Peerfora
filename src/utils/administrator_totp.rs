use totp_rs::{Builder, Secret, Totp};

use crate::errors::app_error::AppError;

pub fn create_administrator_totp(email: &str) -> Result<(Totp, String), AppError> {
    let secret = Secret::generate();
    let totp = Builder::new()
        .with_secret(secret.as_bytes())
        .with_account_name(email)
        .with_issuer(Some("Peerfora"))
        .build()
        .map_err(|_| AppError::Internal)?;
    let provisioning_uri = totp.to_url().map_err(|_| AppError::Internal)?;
    Ok((totp, provisioning_uri))
}

pub fn build_administrator_totp(secret: Vec<u8>, email: &str) -> Result<Totp, AppError> {
    Builder::new()
        .with_secret(Secret::from(secret))
        .with_account_name(email)
        .with_issuer(Some("Peerfora"))
        .build()
        .map_err(|_| AppError::Internal)
}
