use axum::{Router, middleware::from_fn_with_state, routing::get};

use crate::{
    constants::knowledge::{
        ADMIN_KNOWLEDGE_ENTRY_PUBLISH_ROUTE, ADMIN_KNOWLEDGE_PROPOSAL_REVIEW_ROUTE,
        ADMIN_KNOWLEDGE_PROPOSAL_ROUTE, ADMIN_KNOWLEDGE_PROPOSALS_ROUTE,
        ADMIN_KNOWLEDGE_PUBLISH_PERMISSION, ADMIN_KNOWLEDGE_READ_PERMISSION,
        ADMIN_KNOWLEDGE_REVIEW_PERMISSION,
    },
    controllers::admin_knowledge_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route(
            ADMIN_KNOWLEDGE_PROPOSALS_ROUTE,
            get(admin_knowledge_controller::list_knowledge_proposals),
        )
        .route(
            ADMIN_KNOWLEDGE_PROPOSAL_ROUTE,
            get(admin_knowledge_controller::find_knowledge_proposal),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_KNOWLEDGE_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let review_routes = Router::new()
        .route(
            ADMIN_KNOWLEDGE_PROPOSAL_REVIEW_ROUTE,
            axum::routing::post(admin_knowledge_controller::review_knowledge_proposal),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_KNOWLEDGE_REVIEW_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let publish_routes = Router::new()
        .route(
            ADMIN_KNOWLEDGE_ENTRY_PUBLISH_ROUTE,
            axum::routing::post(admin_knowledge_controller::publish_knowledge_entry),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_KNOWLEDGE_PUBLISH_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));
    read_routes.merge(review_routes).merge(publish_routes)
}
