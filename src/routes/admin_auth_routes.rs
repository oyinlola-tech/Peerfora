use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::admin_auth::{
        ADMIN_AUTH_LOGIN_ROUTE, ADMIN_AUTH_LOGOUT_ROUTE, ADMIN_AUTH_ME_ROUTE,
        ADMIN_AUTH_MFA_CONFIRM_ROUTE, ADMIN_AUTH_MFA_ENROLL_ROUTE, ADMIN_AUTH_MFA_VERIFY_ROUTE,
    },
    controllers::admin_auth_controller,
    middlewares::administrator_authentication::require_active_administrator_session,
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        .route(
            ADMIN_AUTH_ME_ROUTE,
            get(admin_auth_controller::current_administrator),
        )
        .route(ADMIN_AUTH_LOGOUT_ROUTE, post(admin_auth_controller::logout))
        .route_layer(from_fn_with_state(
            state,
            require_active_administrator_session,
        ));

    Router::new()
        .route(ADMIN_AUTH_LOGIN_ROUTE, post(admin_auth_controller::login))
        .route(
            ADMIN_AUTH_MFA_ENROLL_ROUTE,
            post(admin_auth_controller::begin_mfa_enrollment),
        )
        .route(
            ADMIN_AUTH_MFA_CONFIRM_ROUTE,
            post(admin_auth_controller::confirm_mfa_enrollment),
        )
        .route(
            ADMIN_AUTH_MFA_VERIFY_ROUTE,
            post(admin_auth_controller::verify_mfa_login),
        )
        .merge(protected_routes)
}
