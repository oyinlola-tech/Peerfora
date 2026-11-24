use axum::{Router, routing::get};

use crate::{
    constants::phase_four::REALTIME_ROUTE, controllers::realtime_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new().route(REALTIME_ROUTE, get(realtime_controller::realtime_events))
}
