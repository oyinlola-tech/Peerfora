use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_CREDENTIAL_DEFAULT_LIFETIME_DAYS, DEVELOPER_CREDENTIAL_MAX_LIFETIME_DAYS,
        DEVELOPER_CREDENTIAL_NAME_MAX_LENGTH, DEVELOPER_CREDENTIAL_NAME_MIN_LENGTH,
    },
    enums::developer::{DeveloperCredentialEnvironment, DeveloperError},
    errors::app_error::AppError,
    models::developer::DeveloperCredentialRecord,
    repositories::developer_application_repository::credentials,
    utils::{machine_credential, one_time_token},
};

pub struct IssuedCredential {
    pub record: DeveloperCredentialRecord,
    pub secret: String,
}

pub async fn issue_developer_credential(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
    name: &str,
    expires_in_days: Option<u16>,
    environment: DeveloperCredentialEnvironment,
) -> Result<IssuedCredential, AppError> {
    let name = name.trim();
    if !(DEVELOPER_CREDENTIAL_NAME_MIN_LENGTH..=DEVELOPER_CREDENTIAL_NAME_MAX_LENGTH)
        .contains(&name.chars().count())
    {
        return Err(DeveloperError::CredentialNameInvalid.into());
    }
    let expires_in_days = expires_in_days.unwrap_or(DEVELOPER_CREDENTIAL_DEFAULT_LIFETIME_DAYS);
    if !(1..=DEVELOPER_CREDENTIAL_MAX_LIFETIME_DAYS).contains(&expires_in_days) {
        return Err(DeveloperError::CredentialLifetimeInvalid.into());
    }
    let secret = machine_credential::create_developer_machine_credential();
    let record = credentials::issue_developer_credential(
        pool,
        credentials::IssueDeveloperCredential {
            owner_account_id,
            application_id,
            credential_id: Uuid::new_v4(),
            token_hash: one_time_token::hash_one_time_token(&secret),
            name: name.to_owned(),
            environment,
            expires_in_days,
        },
    )
    .await?;
    Ok(IssuedCredential { record, secret })
}
