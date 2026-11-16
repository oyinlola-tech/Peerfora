use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_APPLICATION_MAX_REVIEW_REASON_LENGTH,
        DEVELOPER_APPLICATION_MIN_REVIEW_REASON_LENGTH,
    },
    enums::developer::DeveloperError,
    errors::app_error::AppError,
    repositories::developer_application_repository::credentials,
};

pub async fn suspend_developer_application(
    pool: &PgPool,
    application_id: Uuid,
    administrator_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = validate_reason(reason)?;
    credentials::suspend_developer_application(
        pool,
        application_id,
        administrator_id,
        reason,
        correlation_id,
    )
    .await
}

pub async fn revoke_developer_credential_as_administrator(
    pool: &PgPool,
    application_id: Uuid,
    credential_id: Uuid,
    administrator_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let reason = validate_reason(reason)?;
    credentials::revoke_developer_credential_as_administrator(
        pool,
        application_id,
        credential_id,
        administrator_id,
        reason,
        correlation_id,
    )
    .await
}

fn validate_reason(reason: &str) -> Result<&str, AppError> {
    let reason = reason.trim();
    if !(DEVELOPER_APPLICATION_MIN_REVIEW_REASON_LENGTH
        ..=DEVELOPER_APPLICATION_MAX_REVIEW_REASON_LENGTH)
        .contains(&reason.chars().count())
    {
        return Err(DeveloperError::AdministrativeReasonInvalid.into());
    }
    Ok(reason)
}
