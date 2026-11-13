use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::profile_error::ProfileError,
    errors::app_error::AppError,
    models::user_profile::{
        AccountPreferences, OwnUserProfile, PublicUserProfile, UserProfileChanges,
    },
};

pub async fn find_own_user_profile(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<OwnUserProfile>, AppError> {
    sqlx::query_as::<_, OwnUserProfile>(
        "SELECT a.id AS account_id, a.username, a.display_name, p.biography,
            p.avatar_reference, p.banner_reference, p.academic_level_id,
            p.institution_id, p.discipline_id, p.profile_visibility,
            p.show_academic_affiliation
        FROM accounts a JOIN user_profiles p ON p.account_id = a.id
        WHERE a.id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn find_public_user_profile(
    pool: &PgPool,
    username: &str,
) -> Result<Option<PublicUserProfile>, AppError> {
    sqlx::query_as::<_, PublicUserProfile>(
        "SELECT a.username, a.display_name, p.biography, p.avatar_reference,
            p.banner_reference,
            CASE WHEN p.show_academic_affiliation THEN p.academic_level_id END AS academic_level_id,
            CASE WHEN p.show_academic_affiliation THEN i.display_name END AS institution_name,
            CASE WHEN p.show_academic_affiliation THEN d.display_name END AS discipline_name,
            p.show_academic_affiliation,
            a.created_at
        FROM accounts a JOIN user_profiles p ON p.account_id = a.id
        LEFT JOIN institutions i ON i.id = p.institution_id
        LEFT JOIN academic_disciplines d ON d.id = p.discipline_id
        WHERE lower(a.username) = lower($1) AND p.profile_visibility = 'public'",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn update_own_user_profile(
    pool: &PgPool,
    account_id: Uuid,
    changes: UserProfileChanges,
) -> Result<OwnUserProfile, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query("UPDATE accounts SET display_name = COALESCE($2, display_name), updated_at = now() WHERE id = $1")
        .bind(account_id)
        .bind(changes.display_name.as_deref())
        .execute(&mut *transaction)
        .await
        .map_err(map_profile_reference_error)?;
    sqlx::query(
        "UPDATE user_profiles SET biography = COALESCE($2, biography),
            avatar_reference = COALESCE($3, avatar_reference),
            banner_reference = COALESCE($4, banner_reference),
            academic_level_id = COALESCE($5, academic_level_id),
            institution_id = COALESCE($6, institution_id),
            discipline_id = COALESCE($7, discipline_id),
            profile_visibility = COALESCE($8, profile_visibility),
            show_academic_affiliation = COALESCE($9, show_academic_affiliation),
            updated_at = now()
        WHERE account_id = $1",
    )
    .bind(account_id)
    .bind(changes.biography.as_deref())
    .bind(changes.avatar_reference.as_deref())
    .bind(changes.banner_reference.as_deref())
    .bind(changes.academic_level_id.as_deref())
    .bind(changes.institution_id)
    .bind(changes.discipline_id)
    .bind(changes.profile_visibility.as_deref())
    .bind(changes.show_academic_affiliation)
    .execute(&mut *transaction)
    .await
    .map_err(map_profile_reference_error)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    find_own_user_profile(pool, account_id)
        .await?
        .ok_or(ProfileError::ProfileNotFound.into())
}

fn map_profile_reference_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::Database(database_error)
            if database_error.code().as_deref() == Some("23503") =>
        {
            ProfileError::InvalidProfileReference.into()
        }
        _ => AppError::Internal,
    }
}

pub async fn find_account_preferences(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<AccountPreferences>, AppError> {
    sqlx::query_as::<_, AccountPreferences>(
        "SELECT language_tag, country_code::text, notification_preferences,
            content_preferences, personalization_enabled
        FROM account_preferences WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn update_account_preferences(
    pool: &PgPool,
    account_id: Uuid,
    language_tag: Option<&str>,
    country_code: Option<&str>,
    notification_preferences: Option<&serde_json::Value>,
    content_preferences: Option<&serde_json::Value>,
    personalization_enabled: Option<bool>,
) -> Result<AccountPreferences, AppError> {
    sqlx::query(
        "UPDATE account_preferences SET language_tag = COALESCE($2, language_tag),
            country_code = COALESCE($3, country_code),
            notification_preferences = COALESCE($4, notification_preferences),
            content_preferences = COALESCE($5, content_preferences),
            personalization_enabled = COALESCE($6, personalization_enabled), updated_at = now()
        WHERE account_id = $1",
    )
    .bind(account_id)
    .bind(language_tag)
    .bind(country_code)
    .bind(notification_preferences)
    .bind(content_preferences)
    .bind(personalization_enabled)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    find_account_preferences(pool, account_id)
        .await?
        .ok_or(ProfileError::ProfileNotFound.into())
}
