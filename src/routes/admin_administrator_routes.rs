use axum::{Router, middleware::from_fn_with_state, routing::post};

use crate::{
    constants::admin_auth::{
        ADMINISTRATOR_DELETE_ROUTE, ADMINISTRATOR_MFA_RESET_ROUTE, ADMINISTRATOR_REINSTATE_ROUTE,
        ADMINISTRATOR_SESSIONS_REVOKE_ROUTE, ADMINISTRATOR_SUSPEND_ROUTE,
        ADMINISTRATORS_MANAGE_PERMISSION,
    },
    controllers::admin_administrator_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            ADMINISTRATOR_SUSPEND_ROUTE,
            post(admin_administrator_controller::suspend_administrator_account),
        )
        .route(
            ADMINISTRATOR_REINSTATE_ROUTE,
            post(admin_administrator_controller::reinstate_administrator_account),
        )
        .route(
            ADMINISTRATOR_DELETE_ROUTE,
            axum::routing::delete(admin_administrator_controller::delete_administrator_account),
        )
        .route(
            ADMINISTRATOR_SESSIONS_REVOKE_ROUTE,
            axum::routing::delete(
                admin_administrator_controller::revoke_administrator_account_sessions,
            ),
        )
        .route(
            ADMINISTRATOR_MFA_RESET_ROUTE,
            post(admin_administrator_controller::reset_administrator_mfa_factor),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMINISTRATORS_MANAGE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ))
}
