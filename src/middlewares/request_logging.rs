use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

pub async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let response = next.run(request).await;
    info!(%method, %path, status = %response.status(), "HTTP request completed");
    response
}
