use axum::http::{HeaderMap, HeaderValue, header};

use crate::{
    constants::account_auth::{
        SESSION_COOKIE_NAME, SESSION_COOKIE_PATH, SESSION_LIFETIME_SECONDS,
        SOCIAL_AUTH_COOKIE_PATH, SOCIAL_SIGNUP_COOKIE_NAME, SOCIAL_SIGNUP_COOKIE_PATH,
        SOCIAL_SIGNUP_LIFETIME_SECONDS, SOCIAL_STATE_COOKIE_NAME, SOCIAL_STATE_LIFETIME_SECONDS,
    },
    errors::app_error::AppError,
    models::app_state::AppState,
};

pub fn read_auth_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|cookie| {
            let (name, value) = cookie.trim().split_once('=')?;
            (name == cookie_name && !value.is_empty()).then(|| value.to_owned())
        })
}

pub fn read_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
}

pub fn create_session_cookie(state: &AppState, token: &str) -> Result<HeaderValue, AppError> {
    create_cookie(
        state,
        SESSION_COOKIE_NAME,
        token,
        SESSION_COOKIE_PATH,
        SESSION_LIFETIME_SECONDS,
    )
}

pub fn create_oauth_state_cookie(state: &AppState, token: &str) -> Result<HeaderValue, AppError> {
    create_cookie(
        state,
        SOCIAL_STATE_COOKIE_NAME,
        token,
        SOCIAL_AUTH_COOKIE_PATH,
        SOCIAL_STATE_LIFETIME_SECONDS,
    )
}

pub fn create_pending_social_cookie(
    state: &AppState,
    token: &str,
) -> Result<HeaderValue, AppError> {
    create_cookie(
        state,
        SOCIAL_SIGNUP_COOKIE_NAME,
        token,
        SOCIAL_SIGNUP_COOKIE_PATH,
        SOCIAL_SIGNUP_LIFETIME_SECONDS,
    )
}

pub fn clear_auth_cookie(state: &AppState, name: &str) -> Result<HeaderValue, AppError> {
    create_cookie(state, name, "", SESSION_COOKIE_PATH, 0)
}

pub fn clear_oauth_state_cookie(state: &AppState) -> Result<HeaderValue, AppError> {
    create_cookie(
        state,
        SOCIAL_STATE_COOKIE_NAME,
        "",
        SOCIAL_AUTH_COOKIE_PATH,
        0,
    )
}

pub fn clear_pending_social_cookie(state: &AppState) -> Result<HeaderValue, AppError> {
    create_cookie(
        state,
        SOCIAL_SIGNUP_COOKIE_NAME,
        "",
        SOCIAL_SIGNUP_COOKIE_PATH,
        0,
    )
}

fn create_cookie(
    state: &AppState,
    name: &str,
    value: &str,
    path: &str,
    max_age_seconds: u32,
) -> Result<HeaderValue, AppError> {
    let secure_flag = if state.cookie_secure { "; Secure" } else { "" };
    let cookie_value = format!(concat!(
        "{name}={value}; Path={path}; ",
        "HttpOnly; SameSite=Lax; ",
        "Max-Age={max_age_seconds}{secure_flag}"
    ));
    HeaderValue::from_str(&cookie_value).map_err(|_| AppError::Internal)
}
