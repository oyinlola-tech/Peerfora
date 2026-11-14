use std::time::Duration;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderName, Method, header},
    middleware::{from_fn, from_fn_with_state},
};
use tower_http::cors::CorsLayer;

use crate::{
    constants::security::{
        CORS_PREFLIGHT_MAX_AGE_SECONDS, MAX_JSON_REQUEST_BODY_BYTES, REQUEST_CORRELATION_ID_HEADER,
    },
    middlewares::{
        csrf_origin::enforce_request_origin_for_cookie_sessions, request_logging::log_request,
        request_timeout::enforce_request_timeout, security_headers::apply_security_headers,
    },
    models::app_state::AppState,
};

pub fn register_middlewares(app: Router<AppState>, state: AppState) -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_origin(state.allowed_origins.clone())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::ACCEPT, header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true)
        .expose_headers([HeaderName::from_static(REQUEST_CORRELATION_ID_HEADER)])
        .max_age(Duration::from_secs(CORS_PREFLIGHT_MAX_AGE_SECONDS));

    app.layer(cors)
        .layer(from_fn_with_state(
            state.clone(),
            enforce_request_origin_for_cookie_sessions,
        ))
        .layer(from_fn_with_state(state.clone(), enforce_request_timeout))
        .layer(DefaultBodyLimit::max(MAX_JSON_REQUEST_BODY_BYTES))
        .layer(from_fn_with_state(state, apply_security_headers))
        .layer(from_fn(log_request))
}
