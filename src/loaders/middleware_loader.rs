use axum::{Router, middleware::from_fn};

use crate::middlewares::request_logging::log_request;

pub fn register_middlewares(app: Router) -> Router {
    app.layer(from_fn(log_request))
}
