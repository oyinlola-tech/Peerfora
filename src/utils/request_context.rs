use axum::http::HeaderMap;

use crate::constants::security::REQUEST_CORRELATION_ID_HEADER;

pub fn get_request_correlation_id(headers: &HeaderMap) -> Option<&str> {
    headers.get(REQUEST_CORRELATION_ID_HEADER)?.to_str().ok()
}
