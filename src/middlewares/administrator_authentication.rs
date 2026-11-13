use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    constants::admin_auth::ADMIN_SESSION_COOKIE_NAME, enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError, models::app_state::AppState, repositories::admin_auth_repository,
    utils::one_time_token::hash_one_time_token,
};

pub async fn require_active_administrator_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) = read_administrator_session_token(&request) else {
        return AppError::from(AdminAuthError::InvalidSession).into_response();
    };
    let identity = match admin_auth_repository::find_active_administrator_session(
        &state.database,
        &hash_one_time_token(&token),
    )
    .await
    {
        Ok(Some(identity)) => identity,
        Ok(None) => return AppError::from(AdminAuthError::InvalidSession).into_response(),
        Err(_) => {
            return AppError::from(AdminAuthError::AuthenticationUnavailable).into_response();
        }
    };
    request.extensions_mut().insert(identity);
    next.run(request).await
}

fn read_administrator_session_token(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|cookie| {
            let (name, value) = cookie.trim().split_once('=')?;
            (name == ADMIN_SESSION_COOKIE_NAME && !value.is_empty()).then(|| value.to_owned())
        })
}
