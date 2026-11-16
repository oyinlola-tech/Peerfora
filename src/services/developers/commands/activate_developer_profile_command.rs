use sqlx::PgPool;

use crate::{
    constants::developers::{
        DEVELOPER_PROFILE_MAX_DESCRIPTION_LENGTH, DEVELOPER_PROFILE_MAX_NAME_LENGTH,
        DEVELOPER_PROFILE_MIN_NAME_LENGTH,
    },
    enums::developer::DeveloperError,
    errors::app_error::AppError,
    models::developer::DeveloperProfileRecord,
    repositories::developer_profile_repository,
};
use uuid::Uuid;

pub async fn activate_developer_profile(
    pool: &PgPool,
    account_id: Uuid,
    display_name: &str,
    description: &str,
) -> Result<DeveloperProfileRecord, AppError> {
    let display_name = display_name.trim();
    if !(DEVELOPER_PROFILE_MIN_NAME_LENGTH..=DEVELOPER_PROFILE_MAX_NAME_LENGTH)
        .contains(&display_name.chars().count())
    {
        return Err(DeveloperError::ProfileNameInvalid.into());
    }
    if description.chars().count() > DEVELOPER_PROFILE_MAX_DESCRIPTION_LENGTH {
        return Err(DeveloperError::ProfileDescriptionInvalid.into());
    }
    developer_profile_repository::create_developer_profile(
        pool,
        account_id,
        display_name,
        description.trim(),
    )
    .await
}
