use axum::http::StatusCode;

use crate::constants::security::REQUEST_TIMEOUT_MESSAGE;

pub fn get_request_timeout_error_response() -> (StatusCode, &'static str) {
    (StatusCode::GATEWAY_TIMEOUT, REQUEST_TIMEOUT_MESSAGE)
}
