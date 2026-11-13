use axum::{Router, routing::get};

use crate::{
    constants::academic_directory::{
        LIST_ACADEMIC_LEVELS_ROUTE, SEARCH_COUNTRIES_ROUTE, SEARCH_SUBJECTS_ROUTE,
    },
    controllers::academic_directory_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            SEARCH_SUBJECTS_ROUTE,
            get(academic_directory_controller::search_academic_subjects),
        )
        .route(
            SEARCH_COUNTRIES_ROUTE,
            get(academic_directory_controller::search_countries),
        )
        .route(
            LIST_ACADEMIC_LEVELS_ROUTE,
            get(academic_directory_controller::list_academic_levels),
        )
}
