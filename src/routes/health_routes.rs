use axum::{Router, routing::get};

use crate::{
    constants::health::{HEALTH_PATH, READINESS_PATH},
    controllers::health_controller::{show_health, show_readiness},
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(HEALTH_PATH, get(show_health))
        .route(READINESS_PATH, get(show_readiness))
}
