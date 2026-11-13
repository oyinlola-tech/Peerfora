use uuid::Uuid;

use crate::{
    constants::profiles::LANGUAGE_TAG_MAX_BYTES, enums::profile_error::ProfileError,
    errors::app_error::AppError, models::user_profile::AccountPreferences,
    repositories::profile_repository,
};

pub struct AccountPreferenceChanges {
    pub language_tag: Option<String>,
    pub country_code: Option<String>,
    pub notification_preferences: Option<serde_json::Value>,
    pub content_preferences: Option<serde_json::Value>,
    pub personalization_enabled: Option<bool>,
}

pub async fn update_account_preferences(
    pool: &sqlx::PgPool,
    account_id: Uuid,
    changes: AccountPreferenceChanges,
) -> Result<AccountPreferences, AppError> {
    let country_code = changes.country_code.map(|value| value.to_ascii_uppercase());
    if country_code.as_ref().is_some_and(|value| {
        value.len() != 2
            || !value
                .bytes()
                .all(|character| character.is_ascii_alphabetic())
    }) {
        return Err(ProfileError::InvalidPreferenceCountryCode.into());
    }
    if changes.language_tag.as_ref().is_some_and(|value| {
        !crate::utils::language_tag::is_valid_language_tag(value, LANGUAGE_TAG_MAX_BYTES)
    }) {
        return Err(ProfileError::InvalidLanguageTag.into());
    }
    profile_repository::update_account_preferences(
        pool,
        account_id,
        changes.language_tag.as_deref(),
        country_code.as_deref(),
        changes.notification_preferences.as_ref(),
        changes.content_preferences.as_ref(),
        changes.personalization_enabled,
    )
    .await
}
