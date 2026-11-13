use axum::{
    Router,
    routing::{get, put},
};

use crate::{
    constants::communities::{
        COMMUNITIES_ROUTE, COMMUNITY_BY_SLUG_ROUTE, COMMUNITY_MEMBERSHIP_ROUTE,
    },
    controllers::community_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            COMMUNITIES_ROUTE,
            get(community_controller::list_communities)
                .post(community_controller::create_community),
        )
        .route(
            COMMUNITY_BY_SLUG_ROUTE,
            get(community_controller::find_community),
        )
        .route(
            COMMUNITY_MEMBERSHIP_ROUTE,
            put(community_controller::join_community).delete(community_controller::leave_community),
        )
}
