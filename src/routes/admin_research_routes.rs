use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::admin_auth::{
        ADMIN_RESEARCH_PROJECT_BY_ID_ROUTE, ADMIN_RESEARCH_PROJECT_RESTORE_ROUTE,
        ADMIN_RESEARCH_PROJECT_SUSPEND_ROUTE, ADMIN_RESEARCH_PROJECTS_READ_PERMISSION,
        ADMIN_RESEARCH_PROJECTS_RESTORE_PERMISSION, ADMIN_RESEARCH_PROJECTS_ROUTE,
        ADMIN_RESEARCH_PROJECTS_SUSPEND_PERMISSION,
    },
    controllers::admin_research_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route(
            ADMIN_RESEARCH_PROJECTS_ROUTE,
            get(admin_research_controller::list_research_projects),
        )
        .route(
            ADMIN_RESEARCH_PROJECT_BY_ID_ROUTE,
            get(admin_research_controller::show_research_project),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_RESEARCH_PROJECTS_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let suspend_routes = Router::new()
        .route(
            ADMIN_RESEARCH_PROJECT_SUSPEND_ROUTE,
            post(admin_research_controller::suspend_research_project),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_RESEARCH_PROJECTS_SUSPEND_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let restore_routes = Router::new()
        .route(
            ADMIN_RESEARCH_PROJECT_RESTORE_ROUTE,
            post(admin_research_controller::restore_research_project),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_RESEARCH_PROJECTS_RESTORE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));
    read_routes.merge(suspend_routes).merge(restore_routes)
}
