use uuid::Uuid;

use crate::{
    constants::profiles::{
        PROFILE_BIOGRAPHY_MAX_CHARACTERS, PROFILE_DISPLAY_NAME_MAX_CHARACTERS,
        PROFILE_MEDIA_REFERENCE_MAX_BYTES,
    },
    enums::profile_error::ProfileError,
    errors::app_error::AppError,
    models::user_profile::{OwnUserProfile, UserProfileChanges},
    repositories::profile_repository,
    services::academic_directory::queries::list_academic_levels_query,
};

pub type OwnProfileChanges = UserProfileChanges;

pub async fn update_own_profile(
    pool: &sqlx::PgPool,
    account_id: Uuid,
    mut changes: OwnProfileChanges,
) -> Result<OwnUserProfile, AppError> {
    if let Some(supplied_level_id) = changes.academic_level_id.as_deref() {
        changes.academic_level_id = Some(
            list_academic_levels_query::find_canonical_academic_level_id(pool, supplied_level_id)
                .await?
                .ok_or(ProfileError::InvalidProfileReference)?,
        );
    }
    validate_own_profile_changes(&changes)?;
    repository_update_own_profile(pool, account_id, changes).await
}

fn validate_own_profile_changes(changes: &OwnProfileChanges) -> Result<(), AppError> {
    if changes
        .display_name
        .as_ref()
        .is_some_and(|value| value.chars().count() > PROFILE_DISPLAY_NAME_MAX_CHARACTERS)
    {
        return Err(ProfileError::DisplayNameTooLong.into());
    }
    if changes
        .biography
        .as_ref()
        .is_some_and(|value| value.chars().count() > PROFILE_BIOGRAPHY_MAX_CHARACTERS)
    {
        return Err(ProfileError::BiographyTooLong.into());
    }
    if changes
        .profile_visibility
        .as_deref()
        .is_some_and(|value| !matches!(value, "public" | "private"))
    {
        return Err(ProfileError::InvalidProfileVisibility.into());
    }
    if changes
        .avatar_reference
        .as_ref()
        .is_some_and(|value| value.len() > PROFILE_MEDIA_REFERENCE_MAX_BYTES)
        || changes
            .banner_reference
            .as_ref()
            .is_some_and(|value| value.len() > PROFILE_MEDIA_REFERENCE_MAX_BYTES)
    {
        return Err(ProfileError::ProfileReferenceTooLong.into());
    }
    Ok(())
}

async fn repository_update_own_profile(
    pool: &sqlx::PgPool,
    account_id: Uuid,
    changes: OwnProfileChanges,
) -> Result<OwnUserProfile, AppError> {
    profile_repository::update_own_user_profile(pool, account_id, changes).await
}

#[cfg(test)]
mod tests {
    use super::{OwnProfileChanges, validate_own_profile_changes};
    use crate::{enums::profile_error::ProfileError, errors::app_error::AppError};

    fn empty_profile_changes() -> OwnProfileChanges {
        OwnProfileChanges {
            display_name: None,
            biography: None,
            avatar_reference: None,
            banner_reference: None,
            academic_level_id: None,
            institution_id: None,
            discipline_id: None,
            profile_visibility: None,
            show_academic_affiliation: None,
        }
    }

    #[test]
    fn rejects_biography_over_five_hundred_characters() {
        let changes = OwnProfileChanges {
            biography: Some("a".repeat(501)),
            ..empty_profile_changes()
        };

        assert!(matches!(
            validate_own_profile_changes(&changes),
            Err(AppError::Profile(ProfileError::BiographyTooLong))
        ));
    }

    #[test]
    fn rejects_unknown_profile_visibility() {
        let changes = OwnProfileChanges {
            profile_visibility: Some("friends".to_owned()),
            ..empty_profile_changes()
        };

        assert!(matches!(
            validate_own_profile_changes(&changes),
            Err(AppError::Profile(ProfileError::InvalidProfileVisibility))
        ));
    }
}
