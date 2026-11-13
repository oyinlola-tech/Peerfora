use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use uuid::Uuid;

use crate::{
    constants::account_auth::SESSION_COOKIE_NAME,
    controllers::auth_cookie,
    dtos::profile_requests::{UpdateAccountPreferencesRequest, UpdateOwnProfileRequest},
    enums::{account_auth_error::AccountAuthError, profile_error::ProfileError},
    errors::app_error::AppError,
    models::{
        app_state::AppState,
        user_profile::{AccountPreferences, OwnUserProfile, PublicUserProfile},
    },
    services::{
        accounts::queries::account_session_queries,
        profiles::{
            commands::{
                update_account_preferences_command::{self, AccountPreferenceChanges},
                update_own_profile_command::{self, OwnProfileChanges},
            },
            queries::{
                find_account_preferences_query, find_own_profile_query, find_public_profile_query,
            },
        },
    },
};

pub async fn find_own_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<OwnUserProfile>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let profile = find_own_profile_query::find_own_profile(&state.database, account_id)
        .await?
        .ok_or(ProfileError::ProfileNotFound)?;
    Ok(Json(profile))
}

pub async fn update_own_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateOwnProfileRequest>,
) -> Result<Json<OwnUserProfile>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let updated_profile = update_own_profile_command::update_own_profile(
        &state.database,
        account_id,
        OwnProfileChanges {
            display_name: request.display_name,
            biography: request.biography,
            avatar_reference: request.avatar_reference,
            banner_reference: request.banner_reference,
            academic_level_id: request.academic_level_id,
            institution_id: request.institution_id,
            discipline_id: request.discipline_id,
            profile_visibility: request.profile_visibility,
            show_academic_affiliation: request.show_academic_affiliation,
        },
    )
    .await?;
    Ok(Json(updated_profile))
}

pub async fn find_public_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<PublicUserProfile>, AppError> {
    let profile = find_public_profile_query::find_public_profile(&state.database, &username)
        .await?
        .ok_or(ProfileError::ProfileNotFound)?;
    Ok(Json(profile))
}

pub async fn find_own_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AccountPreferences>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let preferences =
        find_account_preferences_query::find_account_preferences(&state.database, account_id)
            .await?
            .ok_or(ProfileError::ProfileNotFound)?;
    Ok(Json(preferences))
}

pub async fn update_own_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateAccountPreferencesRequest>,
) -> Result<Json<AccountPreferences>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let preferences = update_account_preferences_command::update_account_preferences(
        &state.database,
        account_id,
        AccountPreferenceChanges {
            language_tag: request.language_tag,
            country_code: request.country_code,
            notification_preferences: request.notification_preferences,
            content_preferences: request.content_preferences,
            personalization_enabled: request.personalization_enabled,
        },
    )
    .await?;
    Ok(Json(preferences))
}

async fn require_account_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, AppError> {
    let token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(token) = token else {
        return Err(AccountAuthError::SessionInvalid.into());
    };
    account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .map(|account| account.id)
        .ok_or(AccountAuthError::SessionInvalid.into())
}
