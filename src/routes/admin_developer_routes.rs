use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::developers::{
        ADMIN_DEVELOPER_APPLICATION_REVIEW_ROUTE, ADMIN_DEVELOPER_APPLICATION_SUSPEND_ROUTE,
        ADMIN_DEVELOPER_APPLICATIONS_READ_PERMISSION,
        ADMIN_DEVELOPER_APPLICATIONS_REVIEW_PERMISSION, ADMIN_DEVELOPER_APPLICATIONS_ROUTE,
        ADMIN_DEVELOPER_CREDENTIAL_REVOKE_ROUTE, DEVELOPER_APPLICATION_SUSPEND_PERMISSION,
        DEVELOPER_CREDENTIAL_REVOKE_PERMISSION,
    },
    controllers::{developer_controller, developer_credential_controller},
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route(
            ADMIN_DEVELOPER_APPLICATIONS_ROUTE,
            get(developer_controller::list_applications_for_review),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_DEVELOPER_APPLICATIONS_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let review_routes = Router::new()
        .route(
            ADMIN_DEVELOPER_APPLICATION_REVIEW_ROUTE,
            post(developer_controller::review_application),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_DEVELOPER_APPLICATIONS_REVIEW_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let suspend_routes = Router::new()
        .route(
            ADMIN_DEVELOPER_APPLICATION_SUSPEND_ROUTE,
            post(developer_credential_controller::suspend_application),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                DEVELOPER_APPLICATION_SUSPEND_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let revoke_routes = Router::new()
        .route(
            ADMIN_DEVELOPER_CREDENTIAL_REVOKE_ROUTE,
            post(developer_credential_controller::revoke_credential_as_administrator),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                DEVELOPER_CREDENTIAL_REVOKE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));
    read_routes
        .merge(review_routes)
        .merge(suspend_routes)
        .merge(revoke_routes)
}
