use axum::{Router, routing::get};

use crate::{
    constants::profiles::{ACCOUNT_PREFERENCES_ROUTE, OWN_PROFILE_ROUTE, PUBLIC_PROFILE_ROUTE},
    controllers::profile_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            OWN_PROFILE_ROUTE,
            get(profile_controller::find_own_profile).patch(profile_controller::update_own_profile),
        )
        .route(
            PUBLIC_PROFILE_ROUTE,
            get(profile_controller::find_public_profile),
        )
        .route(
            ACCOUNT_PREFERENCES_ROUTE,
            get(profile_controller::find_own_preferences)
                .patch(profile_controller::update_own_preferences),
        )
}
