use axum::{
    extract::Request,
    http::{Method, header},
    middleware::Next,
    response::Response,
};

use crate::{
    constants::{account_auth::SESSION_COOKIE_NAME, admin_auth::ADMIN_SESSION_COOKIE_NAME},
    errors::app_error::AppError,
    models::app_state::AppState,
};

pub async fn enforce_request_origin_for_cookie_sessions(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let request_origin = get_single_request_origin(request.headers())?;
    if let Some(origin) = request_origin
        && !state
            .allowed_origins
            .iter()
            .any(|allowed| allowed == origin)
    {
        return Err(AppError::RequestOriginDenied);
    }

    if is_state_changing_method(request.method())
        && request_uses_peerfora_session_cookie(request.headers())
        && request_origin.is_none()
    {
        return Err(AppError::RequestOriginDenied);
    }

    Ok(next.run(request).await)
}

fn get_single_request_origin(
    headers: &axum::http::HeaderMap,
) -> Result<Option<&axum::http::HeaderValue>, AppError> {
    let mut origins = headers.get_all(header::ORIGIN).iter();
    let origin = origins.next();
    if origins.next().is_some() {
        return Err(AppError::RequestOriginDenied);
    }
    Ok(origin)
}

fn is_state_changing_method(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn request_uses_peerfora_session_cookie(headers: &axum::http::HeaderMap) -> bool {
    headers.get_all(header::COOKIE).iter().any(|cookie_header| {
        cookie_header.to_str().is_ok_and(|cookies| {
            cookies.split(';').any(|cookie| {
                cookie.trim().split_once('=').is_some_and(|(name, _)| {
                    name == SESSION_COOKIE_NAME || name == ADMIN_SESSION_COOKIE_NAME
                })
            })
        })
    })
}
