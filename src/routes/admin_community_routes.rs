use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::admin_auth::{
        ADMIN_COMMUNITIES_MANAGE_PERMISSION, ADMIN_COMMUNITIES_READ_PERMISSION,
        ADMIN_COMMUNITIES_ROUTE, ADMIN_COMMUNITY_BY_ID_ROUTE, ADMIN_COMMUNITY_RESTORE_ROUTE,
        ADMIN_COMMUNITY_SUSPEND_ROUTE,
    },
    controllers::admin_community_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route(
            ADMIN_COMMUNITIES_ROUTE,
            get(admin_community_controller::list_communities),
        )
        .route(
            ADMIN_COMMUNITY_BY_ID_ROUTE,
            get(admin_community_controller::show_community),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_COMMUNITIES_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let management_routes = Router::new()
        .route(
            ADMIN_COMMUNITY_SUSPEND_ROUTE,
            post(admin_community_controller::suspend_community_access),
        )
        .route(
            ADMIN_COMMUNITY_RESTORE_ROUTE,
            post(admin_community_controller::restore_community_access),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_COMMUNITIES_MANAGE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));
    read_routes.merge(management_routes)
}
