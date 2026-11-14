use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::admin_auth::{
        ADMIN_AUDIT_LOGS_READ_PERMISSION, ADMIN_AUDIT_LOGS_ROUTE, ADMIN_AUTH_LOGIN_ROUTE,
        ADMIN_AUTH_LOGOUT_ROUTE, ADMIN_AUTH_ME_ROUTE, ADMIN_AUTH_MFA_CONFIRM_ROUTE,
        ADMIN_AUTH_MFA_ENROLL_ROUTE, ADMIN_AUTH_MFA_RECOVERY_CODES_REGENERATE_ROUTE,
        ADMIN_AUTH_MFA_VERIFY_ROUTE, ADMIN_AUTH_PASSWORD_CHANGE_ROUTE,
        ADMIN_AUTH_PASSWORD_RESET_CONFIRM_ROUTE, ADMIN_AUTH_PASSWORD_RESET_REQUEST_ROUTE,
        ADMIN_INVITATION_ACCEPT_ROUTE, ADMIN_INVITATIONS_MANAGE_PERMISSION,
        ADMIN_INVITATIONS_ROUTE, ADMIN_OVERVIEW_READ_PERMISSION, ADMIN_OVERVIEW_ROUTE,
        ADMIN_SESSION_BY_ID_ROUTE, ADMIN_SESSIONS_ROUTE,
    },
    controllers::{
        admin_auth_controller, admin_auth_recovery_controller, admin_invitation_controller,
    },
    middlewares::administrator_authentication::{
        AdministratorPermissionGuard, require_active_administrator_session,
        require_administrator_permission,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        .route(
            ADMIN_AUTH_ME_ROUTE,
            get(admin_auth_controller::current_administrator),
        )
        .route(ADMIN_AUTH_LOGOUT_ROUTE, post(admin_auth_controller::logout))
        .route(
            ADMIN_AUTH_PASSWORD_CHANGE_ROUTE,
            post(admin_auth_controller::change_own_password),
        )
        .route(
            ADMIN_AUTH_MFA_RECOVERY_CODES_REGENERATE_ROUTE,
            post(admin_auth_controller::regenerate_recovery_codes),
        )
        .route(
            ADMIN_SESSIONS_ROUTE,
            get(admin_auth_controller::current_administrator_sessions),
        )
        .route(
            ADMIN_SESSION_BY_ID_ROUTE,
            axum::routing::delete(admin_auth_controller::revoke_current_administrator_session),
        )
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let audit_routes = Router::new()
        .route(
            ADMIN_AUDIT_LOGS_ROUTE,
            get(admin_auth_controller::list_audit_logs),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_AUDIT_LOGS_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let overview_routes = Router::new()
        .route(
            ADMIN_OVERVIEW_ROUTE,
            get(admin_auth_controller::show_overview),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_OVERVIEW_READ_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));
    let invitation_management_routes = Router::new()
        .route(
            ADMIN_INVITATIONS_ROUTE,
            post(admin_invitation_controller::create_invitation),
        )
        .route_layer(from_fn_with_state(
            AdministratorPermissionGuard::new(
                state.database.clone(),
                ADMIN_INVITATIONS_MANAGE_PERMISSION,
            ),
            require_administrator_permission,
        ))
        .route_layer(from_fn_with_state(
            state.clone(),
            require_active_administrator_session,
        ));

    Router::new()
        .route(ADMIN_AUTH_LOGIN_ROUTE, post(admin_auth_controller::login))
        .route(
            ADMIN_AUTH_PASSWORD_RESET_REQUEST_ROUTE,
            post(admin_auth_recovery_controller::request_password_reset),
        )
        .route(
            ADMIN_AUTH_PASSWORD_RESET_CONFIRM_ROUTE,
            post(admin_auth_recovery_controller::confirm_password_reset),
        )
        .route(
            ADMIN_INVITATION_ACCEPT_ROUTE,
            post(admin_invitation_controller::accept_invitation),
        )
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
        .merge(audit_routes)
        .merge(overview_routes)
        .merge(invitation_management_routes)
}
