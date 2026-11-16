use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_CREDENTIAL_DEFAULT_LIFETIME_DAYS, DEVELOPER_CREDENTIAL_MAX_LIFETIME_DAYS,
    },
    enums::developer::{DeveloperCredentialEnvironment, DeveloperError},
    errors::app_error::AppError,
    models::developer::DeveloperCredentialRecord,
    repositories::developer_application_repository::credentials,
    utils::{machine_credential, one_time_token},
};

pub struct RotatedCredential {
    pub record: DeveloperCredentialRecord,
    pub secret: String,
}

pub async fn rotate_developer_credential(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
    previous_credential_id: Uuid,
    expires_in_days: Option<u16>,
    environment: DeveloperCredentialEnvironment,
) -> Result<RotatedCredential, AppError> {
    let expires_in_days = expires_in_days.unwrap_or(DEVELOPER_CREDENTIAL_DEFAULT_LIFETIME_DAYS);
    if !(1..=DEVELOPER_CREDENTIAL_MAX_LIFETIME_DAYS).contains(&expires_in_days) {
        return Err(DeveloperError::CredentialLifetimeInvalid.into());
    }
    let secret = machine_credential::create_developer_machine_credential();
    let record = credentials::rotate_developer_credential(
        pool,
        credentials::RotateDeveloperCredential {
            owner_account_id,
            application_id,
            previous_credential_id,
            new_credential_id: Uuid::new_v4(),
            new_token_hash: one_time_token::hash_one_time_token(&secret),
            expires_in_days,
            environment,
        },
    )
    .await?;
    Ok(RotatedCredential { record, secret })
}
