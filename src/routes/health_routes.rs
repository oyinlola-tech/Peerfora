use axum::{Router, routing::get};

use crate::{
    constants::health::HEALTH_PATH, controllers::health_controller::show_health,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new().route(HEALTH_PATH, get(show_health))
}
