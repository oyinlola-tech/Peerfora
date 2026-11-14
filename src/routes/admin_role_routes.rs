use axum::{Router, middleware::from_fn_with_state, routing::get};

use crate::{
    constants::admin_auth::{
        ADMIN_ADMINISTRATOR_PERMISSIONS_ROUTE, ADMIN_ADMINISTRATOR_ROLES_ROUTE,
        ADMIN_PERMISSIONS_ASSIGN_PERMISSION, ADMIN_PERMISSIONS_READ_PERMISSION,
        ADMIN_PERMISSIONS_ROUTE, ADMIN_ROLE_BY_ID_ROUTE, ADMIN_ROLE_PERMISSIONS_ROUTE,
        ADMIN_ROLES_ASSIGN_PERMISSION, ADMIN_ROLES_CREATE_PERMISSION,
        ADMIN_ROLES_DELETE_PERMISSION, ADMIN_ROLES_READ_PERMISSION, ADMIN_ROLES_ROUTE,
        ADMIN_ROLES_UPDATE_PERMISSION,
    },
    controllers::admin_role_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let read_routes = Router::new()
        .route(ADMIN_ROLES_ROUTE, get(admin_role_controller::list_roles))
        .route(
            ADMIN_ROLE_BY_ID_ROUTE,
            get(admin_role_controller::show_role),
        )
        .route(
            ADMIN_ADMINISTRATOR_ROLES_ROUTE,
            get(admin_role_controller::list_administrator_roles),
        )
        .route(
            ADMIN_ADMINISTRATOR_PERMISSIONS_ROUTE,
            get(admin_role_controller::list_administrator_permissions),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(state.database.clone(), ADMIN_ROLES_READ_PERMISSION),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let permission_catalog = Router::new()
        .route(
            ADMIN_PERMISSIONS_ROUTE,
            get(admin_role_controller::list_permissions),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_PERMISSIONS_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let role_creation = Router::new()
        .route(
            ADMIN_ROLES_ROUTE,
            axum::routing::post(admin_role_controller::create_role),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_ROLES_CREATE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let role_updates = Router::new()
        .route(
            ADMIN_ROLE_BY_ID_ROUTE,
            axum::routing::patch(admin_role_controller::update_role),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_ROLES_UPDATE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let role_deletion = Router::new()
        .route(
            ADMIN_ROLE_BY_ID_ROUTE,
            axum::routing::delete(admin_role_controller::delete_role),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_ROLES_DELETE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let permission_read = Router::new()
        .route(
            ADMIN_ROLE_PERMISSIONS_ROUTE,
            get(admin_role_controller::show_role),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_PERMISSIONS_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let permission_update = Router::new()
        .route(
            ADMIN_ROLE_PERMISSIONS_ROUTE,
            axum::routing::put(admin_role_controller::replace_role_permissions),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_PERMISSIONS_ASSIGN_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let administrator_role_updates = Router::new()
        .route(
            ADMIN_ADMINISTRATOR_ROLES_ROUTE,
            axum::routing::put(admin_role_controller::replace_administrator_roles),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_ROLES_ASSIGN_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));

    read_routes
        .merge(permission_catalog)
        .merge(role_creation)
        .merge(role_updates)
        .merge(role_deletion)
        .merge(permission_read)
        .merge(permission_update)
        .merge(administrator_role_updates)
}
