use axum::http::StatusCode;

use crate::constants::security::CSRF_ORIGIN_DENIED_MESSAGE;

pub fn get_request_origin_error_response() -> (StatusCode, &'static str) {
    (StatusCode::FORBIDDEN, CSRF_ORIGIN_DENIED_MESSAGE)
}
