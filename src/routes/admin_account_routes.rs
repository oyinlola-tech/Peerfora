use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::admin_auth::{
        ADMIN_ACCOUNT_BY_ID_ROUTE, ADMIN_ACCOUNT_REINSTATE_ROUTE,
        ADMIN_ACCOUNT_SESSIONS_REVOKE_PERMISSION, ADMIN_ACCOUNT_SESSIONS_REVOKE_ROUTE,
        ADMIN_ACCOUNT_SUSPEND_PERMISSION, ADMIN_ACCOUNT_SUSPEND_ROUTE,
        ADMIN_ACCOUNTS_READ_PERMISSION, ADMIN_ACCOUNTS_ROUTE,
    },
    controllers::admin_account_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let database = state.database.clone();
    let account_read_routes = Router::new()
        .route(
            ADMIN_ACCOUNTS_ROUTE,
            get(admin_account_controller::list_accounts),
        )
        .route(
            ADMIN_ACCOUNT_BY_ID_ROUTE,
            get(admin_account_controller::show_account),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_ACCOUNTS_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let account_enforcement_routes = Router::new()
        .route(
            ADMIN_ACCOUNT_SUSPEND_ROUTE,
            post(admin_account_controller::suspend_account_access),
        )
        .route(
            ADMIN_ACCOUNT_REINSTATE_ROUTE,
            post(admin_account_controller::reinstate_account_access),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(database.clone(), ADMIN_ACCOUNT_SUSPEND_PERMISSION),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let account_session_routes = Router::new()
        .route(
            ADMIN_ACCOUNT_SESSIONS_REVOKE_ROUTE,
            post(admin_account_controller::revoke_account_sessions),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(database, ADMIN_ACCOUNT_SESSIONS_REVOKE_PERMISSION),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));
    account_read_routes
        .merge(account_enforcement_routes)
        .merge(account_session_routes)
}
