use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_APPLICATION_MAX_DESCRIPTION_LENGTH, DEVELOPER_APPLICATION_MAX_NAME_LENGTH,
        DEVELOPER_APPLICATION_MAX_SCOPES, DEVELOPER_APPLICATION_MIN_NAME_LENGTH,
    },
    enums::developer::{DeveloperApplicationType, DeveloperError},
    errors::app_error::AppError,
    models::developer::DeveloperApplicationRecord,
    repositories::developer_application_repository,
};

pub async fn create_developer_application(
    pool: &PgPool,
    owner_account_id: Uuid,
    name: &str,
    description: &str,
    application_type: DeveloperApplicationType,
    requested_scopes: &[crate::enums::developer::DeveloperApplicationScope],
) -> Result<DeveloperApplicationRecord, AppError> {
    let name = name.trim();
    if !(DEVELOPER_APPLICATION_MIN_NAME_LENGTH..=DEVELOPER_APPLICATION_MAX_NAME_LENGTH)
        .contains(&name.chars().count())
    {
        return Err(DeveloperError::ApplicationNameInvalid.into());
    }
    if description.chars().count() > DEVELOPER_APPLICATION_MAX_DESCRIPTION_LENGTH {
        return Err(DeveloperError::ApplicationDescriptionInvalid.into());
    }
    if requested_scopes.is_empty() {
        return Err(DeveloperError::NoRequestedScopes.into());
    }
    if requested_scopes.len() > DEVELOPER_APPLICATION_MAX_SCOPES {
        return Err(DeveloperError::TooManyRequestedScopes.into());
    }
    if requested_scopes.iter().collect::<HashSet<_>>().len() != requested_scopes.len() {
        return Err(DeveloperError::DuplicateRequestedScope.into());
    }
    developer_application_repository::create_developer_application(
        pool,
        owner_account_id,
        name,
        description.trim(),
        application_type.as_database_value(),
        requested_scopes,
    )
    .await
}
