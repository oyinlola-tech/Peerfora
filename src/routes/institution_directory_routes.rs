use axum::{Router, routing::get};

use crate::{
    constants::institutions::SEARCH_INSTITUTIONS_ROUTE,
    controllers::institution_directory_controller, models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new().route(
        SEARCH_INSTITUTIONS_ROUTE,
        get(institution_directory_controller::search_institutions),
    )
}
