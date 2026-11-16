use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::moderation::{
        ADMIN_REPORT_ASSIGN_ROUTE, ADMIN_REPORT_READ_PERMISSION, ADMIN_REPORT_RESOLVE_PERMISSION,
        ADMIN_REPORT_RESOLVE_ROUTE, ADMIN_REPORTS_ROUTE,
    },
    controllers::moderation_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route(
            ADMIN_REPORTS_ROUTE,
            get(moderation_controller::list_administrative_reports),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(state.database.clone(), ADMIN_REPORT_READ_PERMISSION),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let resolve_routes = Router::new()
        .route(
            ADMIN_REPORT_ASSIGN_ROUTE,
            post(moderation_controller::assign_administrative_report),
        )
        .route(
            ADMIN_REPORT_RESOLVE_ROUTE,
            post(moderation_controller::resolve_administrative_report),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_REPORT_RESOLVE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));
    read_routes.merge(resolve_routes)
}
