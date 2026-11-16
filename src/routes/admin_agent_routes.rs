use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post, put},
};

use crate::{
    constants::agents::{
        ADMIN_AGENT_LIFECYCLE_ROUTE, ADMIN_AGENT_RESEARCH_QUOTA_ROUTE, ADMIN_AGENT_REVIEW_ROUTE,
        ADMIN_AGENTS_MANAGE_PERMISSION, ADMIN_AGENTS_READ_PERMISSION,
        ADMIN_AGENTS_REVIEW_PERMISSION, ADMIN_AGENTS_ROUTE,
    },
    controllers::admin_agent_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let list_routes = Router::new()
        .route(
            ADMIN_AGENTS_ROUTE,
            get(admin_agent_controller::list_agents_for_review),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(state.database.clone(), ADMIN_AGENTS_READ_PERMISSION),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let review_routes = Router::new()
        .route(
            ADMIN_AGENT_REVIEW_ROUTE,
            post(admin_agent_controller::review_agent_registration),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_AGENTS_REVIEW_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let lifecycle_routes = Router::new()
        .route(
            ADMIN_AGENT_LIFECYCLE_ROUTE,
            post(admin_agent_controller::administrate_agent_lifecycle),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_AGENTS_MANAGE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let quota_routes = Router::new()
        .route(
            ADMIN_AGENT_RESEARCH_QUOTA_ROUTE,
            put(admin_agent_controller::update_agent_research_quota),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_AGENTS_MANAGE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));
    list_routes
        .merge(review_routes)
        .merge(lifecycle_routes)
        .merge(quota_routes)
}
