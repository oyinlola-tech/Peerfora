use axum::{Router, routing::get};

use crate::{
    constants::knowledge::{
        KNOWLEDGE_ENTRIES_ROUTE, KNOWLEDGE_ENTRY_ROUTE, KNOWLEDGE_PROPOSALS_ROUTE,
    },
    controllers::knowledge_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            KNOWLEDGE_PROPOSALS_ROUTE,
            axum::routing::post(knowledge_controller::submit_knowledge_proposal),
        )
        .route(
            KNOWLEDGE_ENTRIES_ROUTE,
            get(knowledge_controller::list_public_knowledge_entries),
        )
        .route(
            KNOWLEDGE_ENTRY_ROUTE,
            get(knowledge_controller::find_public_knowledge_entry),
        )
}
