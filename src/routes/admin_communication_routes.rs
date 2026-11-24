use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{post, put},
};

use crate::{
    constants::phase_four::{
        ADMIN_CONVERSATION_CLOSE_ROUTE, ADMIN_CONVERSATION_RESTRICT_ROUTE,
        ADMIN_MEDIA_QUARANTINE_PERMISSION, ADMIN_MEDIA_ROUTE,
        ADMIN_MESSAGING_ENFORCEMENT_MANAGE_PERMISSION,
    },
    controllers::admin_communication_controller,
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let media_routes = Router::new()
        .route(
            ADMIN_MEDIA_ROUTE,
            put(admin_communication_controller::review_media),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_MEDIA_QUARANTINE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));

    let communication_routes = Router::new()
        .route(
            ADMIN_CONVERSATION_RESTRICT_ROUTE,
            post(admin_communication_controller::restrict_conversation),
        )
        .route(
            ADMIN_CONVERSATION_CLOSE_ROUTE,
            post(admin_communication_controller::close_conversation),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_MESSAGING_ENFORCEMENT_MANAGE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));

    media_routes.merge(communication_routes)
}
