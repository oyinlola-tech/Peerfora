use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, header},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;

use crate::{
    constants::account_auth::SOCIAL_SIGNUP_PAGE_PATH,
    controllers::auth_cookie,
    dtos::auth_responses::AccountAuthResponse,
    enums::account_auth_error::AccountAuthError,
    enums::social_provider::SocialProvider,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::accounts::commands::{
        complete_social_signup_command::{self, SocialSignupDetails},
        finish_social_sign_in_command,
        social_auth_types::SocialAuthorizationResult,
        start_social_sign_in_command,
    },
    validations::account_auth_validation::{parse_date_of_birth, validate_country_code},
};

#[derive(Deserialize)]
pub struct SocialCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteSocialSignupRequest {
    pub username: String,
    pub date_of_birth: String,
    pub country_code: String,
    pub accepted_terms: bool,
}

pub async fn start_social_sign_in(
    State(state): State<AppState>,
    Path(provider_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let provider = parse_social_provider(&provider_name)?;
    let settings = get_provider_settings(&state, &provider)?;
    let authorization =
        start_social_sign_in_command::start_social_sign_in(&state.database, &provider, settings)
            .await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        auth_cookie::create_oauth_state_cookie(&state, &authorization.state)?,
    );
    Ok((headers, Redirect::to(&authorization.authorization_url)))
}

pub async fn finish_social_sign_in(
    State(state): State<AppState>,
    Path(provider_name): Path<String>,
    Query(query): Query<SocialCallbackQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let saved_state = auth_cookie::read_auth_cookie(
        &headers,
        crate::constants::account_auth::SOCIAL_STATE_COOKIE_NAME,
    )
    .ok_or(AccountAuthError::SocialStateInvalid)?;
    if saved_state != query.state {
        return Err(AccountAuthError::SocialStateInvalid.into());
    }
    let provider = parse_social_provider(&provider_name)?;
    let settings = get_provider_settings(&state, &provider)?;
    let result = finish_social_sign_in_command::finish_social_sign_in(
        &state.database,
        &provider,
        settings,
        &query.state,
        &query.code,
    )
    .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        auth_cookie::clear_oauth_state_cookie(&state)?,
    );
    match result {
        SocialAuthorizationResult::SignedIn {
            account: _,
            session_token,
        } => {
            response_headers.append(
                header::SET_COOKIE,
                auth_cookie::create_session_cookie(&state, &session_token)?,
            );
            Ok((
                response_headers,
                Redirect::to(&format!("{}/", state.public_url.trim_end_matches('/'))),
            )
                .into_response())
        }
        SocialAuthorizationResult::NeedsBirthday { onboarding_token } => {
            response_headers.append(
                header::SET_COOKIE,
                auth_cookie::create_pending_social_cookie(&state, &onboarding_token)?,
            );
            Ok((
                response_headers,
                Redirect::to(&format!(
                    "{}{SOCIAL_SIGNUP_PAGE_PATH}",
                    state.public_url.trim_end_matches('/')
                )),
            )
                .into_response())
        }
    }
}

pub async fn complete_social_signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CompleteSocialSignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    let onboarding_token = auth_cookie::read_auth_cookie(
        &headers,
        crate::constants::account_auth::SOCIAL_SIGNUP_COOKIE_NAME,
    )
    .ok_or(AccountAuthError::SocialSignupExpired)?;
    let date_of_birth = parse_date_of_birth(&request.date_of_birth)?;
    let country_code = validate_country_code(&request.country_code)?;
    let (account, session_token) = complete_social_signup_command::complete_social_signup(
        &state.database,
        &state.country_policies,
        SocialSignupDetails {
            onboarding_token,
            username: request.username,
            date_of_birth,
            country_code,
            accepted_terms: request.accepted_terms,
        },
    )
    .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        auth_cookie::clear_pending_social_cookie(&state)?,
    );
    response_headers.append(
        header::SET_COOKIE,
        auth_cookie::create_session_cookie(&state, &session_token)?,
    );
    Ok((response_headers, Json(AccountAuthResponse::from(account))))
}

fn parse_social_provider(provider_name: &str) -> Result<SocialProvider, AppError> {
    SocialProvider::from_path_segment(provider_name)
        .ok_or(AccountAuthError::SocialProviderUnsupported.into())
}

fn get_provider_settings<'a>(
    state: &'a AppState,
    provider: &SocialProvider,
) -> Result<&'a crate::configs::settings::OidcProviderSettings, AppError> {
    let provider_settings = match provider {
        SocialProvider::Google => state.google_oidc.as_ref(),
        SocialProvider::Apple => state.apple_oidc.as_ref(),
    };
    provider_settings.ok_or(AccountAuthError::SocialProviderNotConfigured.into())
}
