use axum::http::HeaderMap;
use uuid::Uuid;

use crate::{
    constants::account_auth::SESSION_COOKIE_NAME, controllers::auth_cookie,
    enums::account_auth_error::AccountAuthError, errors::app_error::AppError,
    models::app_state::AppState, services::accounts::queries::account_session_queries,
};

pub async fn require_authenticated_account_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Uuid, AppError> {
    find_optional_account_id(state, headers)
        .await?
        .ok_or(AccountAuthError::SessionInvalid.into())
}

pub async fn find_optional_account_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<Uuid>, AppError> {
    let token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(token) = token else {
        return Ok(None);
    };
    let account = account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .ok_or(AccountAuthError::SessionInvalid)?;
    Ok(Some(account.id))
}
