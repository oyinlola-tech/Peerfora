use axum::{Router, middleware::from_fn};

use crate::{middlewares::request_logging::log_request, models::app_state::AppState};

pub fn register_middlewares(app: Router<AppState>) -> Router<AppState> {
    app.layer(from_fn(log_request))
}
