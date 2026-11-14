use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header},
    response::IntoResponse,
};

use crate::{
    constants::account_auth::SESSION_COOKIE_NAME,
    controllers::auth_cookie,
    dtos::{auth_requests::LoginAccountRequest, auth_responses::AccountAuthResponse},
    enums::account_auth_error::AccountAuthError,
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::access_control::commands::enforce_authentication_rate_limit_command::enforce_authentication_rate_limit,
    services::accounts::{
        commands::{end_account_session_command, login_account_command},
        queries::account_session_queries,
    },
    validations::account_auth_validation::validate_email_address,
};

pub async fn login_account(
    State(state): State<AppState>,
    Json(request): Json<LoginAccountRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_email_address(&request.email)?;
    enforce_authentication_rate_limit(
        &state.database,
        AuthenticationRateLimitOperation::AccountLogin,
        &request.email,
        &state.authentication_rate_limit_hash_key,
    )
    .await?;
    let session =
        login_account_command::login_account(&state.database, &request.email, request.password)
            .await?;
    let cookie = auth_cookie::create_session_cookie(&state, &session.token)?;
    Ok((
        [(header::SET_COOKIE, cookie)],
        Json(AccountAuthResponse::from(session.account)),
    ))
}

pub async fn get_current_account(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AccountAuthResponse>, AppError> {
    let token = auth_cookie::read_bearer_token(&headers)
        .or_else(|| auth_cookie::read_auth_cookie(&headers, SESSION_COOKIE_NAME))
        .ok_or(AccountAuthError::SessionInvalid)?;
    let account = account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .ok_or(AccountAuthError::SessionInvalid)?;
    Ok(Json(AccountAuthResponse::from(account)))
}

pub async fn logout_account(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    if let Some(token) = auth_cookie::read_bearer_token(&headers)
        .or_else(|| auth_cookie::read_auth_cookie(&headers, SESSION_COOKIE_NAME))
    {
        end_account_session_command::end_account_session(&state.database, &token).await?;
    }
    let clear_cookie = auth_cookie::clear_auth_cookie(&state, SESSION_COOKIE_NAME)?;
    Ok((
        [(header::SET_COOKIE, clear_cookie)],
        axum::http::StatusCode::NO_CONTENT,
    ))
}
