use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    constants::agents::{AGENTS_ROUTE, MY_AGENTS_ROUTE},
    controllers::agent_identity_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            AGENTS_ROUTE,
            post(agent_identity_controller::register_agent),
        )
        .route(
            MY_AGENTS_ROUTE,
            get(agent_identity_controller::list_my_agents),
        )
}
