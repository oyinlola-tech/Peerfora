use axum::{
    Router, extract::DefaultBodyLimit, middleware::from_fn, middleware::from_fn_with_state,
};

use crate::{
    constants::security::MAX_JSON_REQUEST_BODY_BYTES,
    middlewares::{request_logging::log_request, security_headers::apply_security_headers},
    models::app_state::AppState,
};

pub fn register_middlewares(app: Router<AppState>, state: AppState) -> Router<AppState> {
    app.layer(DefaultBodyLimit::max(MAX_JSON_REQUEST_BODY_BYTES))
        .layer(from_fn_with_state(state, apply_security_headers))
        .layer(from_fn(log_request))
}
