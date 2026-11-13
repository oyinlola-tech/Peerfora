use axum::{
    extract::State,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

use crate::{constants::security, models::app_state::AppState};

pub async fn apply_security_headers(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static(security::X_CONTENT_TYPE_OPTIONS_HEADER),
        HeaderValue::from_static(security::X_CONTENT_TYPE_OPTIONS_VALUE),
    );
    headers.insert(
        HeaderName::from_static(security::REFERRER_POLICY_HEADER),
        HeaderValue::from_static(security::REFERRER_POLICY_VALUE),
    );
    headers.insert(
        HeaderName::from_static(security::X_FRAME_OPTIONS_HEADER),
        HeaderValue::from_static(security::X_FRAME_OPTIONS_VALUE),
    );
    headers.insert(
        HeaderName::from_static(security::CACHE_CONTROL_HEADER),
        HeaderValue::from_static(security::CACHE_CONTROL_VALUE),
    );
    headers.insert(
        HeaderName::from_static(security::CONTENT_SECURITY_POLICY_HEADER),
        HeaderValue::from_static(security::CONTENT_SECURITY_POLICY_VALUE),
    );
    if state.production_environment {
        headers.insert(
            HeaderName::from_static(security::STRICT_TRANSPORT_SECURITY_HEADER),
            HeaderValue::from_static(security::STRICT_TRANSPORT_SECURITY_VALUE),
        );
    }
    response
}
