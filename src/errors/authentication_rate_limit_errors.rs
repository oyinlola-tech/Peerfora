use axum::http::StatusCode;

use crate::constants::authentication_rate_limits::AUTHENTICATION_RATE_LIMIT_MESSAGE;

pub fn get_authentication_rate_limit_error_response() -> (StatusCode, &'static str) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        AUTHENTICATION_RATE_LIMIT_MESSAGE,
    )
}
