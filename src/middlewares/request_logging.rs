use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::info;

use crate::constants::security::REQUEST_CORRELATION_ID_HEADER;

pub async fn log_request(mut request: Request, next: Next) -> Response {
    let correlation_id = uuid::Uuid::new_v4().to_string();
    if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
        request
            .headers_mut()
            .insert(REQUEST_CORRELATION_ID_HEADER, header_value.clone());
    }
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
        response
            .headers_mut()
            .insert(REQUEST_CORRELATION_ID_HEADER, header_value);
    }
    info!(%correlation_id, %method, %path, status = %response.status(), "HTTP request completed");
    response
}
