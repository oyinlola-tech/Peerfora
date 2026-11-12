use axum::{Router, routing::get};

use crate::{constants::health::HEALTH_PATH, controllers::health_controller::show_health};

pub fn build_routes() -> Router {
    Router::new().route(HEALTH_PATH, get(show_health))
}
