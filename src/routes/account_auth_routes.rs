use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    constants::account_auth::{
        COMPLETE_SOCIAL_SIGNUP_ROUTE, CONFIRM_PASSWORD_RESET_ROUTE, CURRENT_ACCOUNT_ROUTE,
        LOGIN_ACCOUNT_ROUTE, LOGOUT_ACCOUNT_ROUTE, REGISTER_ACCOUNT_ROUTE,
        REQUEST_PASSWORD_RESET_ROUTE, SOCIAL_SIGNIN_CALLBACK_ROUTE, SOCIAL_SIGNIN_START_ROUTE,
        VERIFY_EMAIL_ROUTE,
    },
    controllers::{
        account_recovery_controller, account_registration_controller, account_session_controller,
        social_auth_controller,
    },
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            REGISTER_ACCOUNT_ROUTE,
            post(account_registration_controller::register_account),
        )
        .route(
            LOGIN_ACCOUNT_ROUTE,
            post(account_session_controller::login_account),
        )
        .route(
            VERIFY_EMAIL_ROUTE,
            get(account_recovery_controller::verify_account_email),
        )
        .route(
            REQUEST_PASSWORD_RESET_ROUTE,
            post(account_recovery_controller::request_password_reset),
        )
        .route(
            CONFIRM_PASSWORD_RESET_ROUTE,
            post(account_recovery_controller::confirm_password_reset),
        )
        .route(
            SOCIAL_SIGNIN_START_ROUTE,
            get(social_auth_controller::start_social_sign_in),
        )
        .route(
            SOCIAL_SIGNIN_CALLBACK_ROUTE,
            get(social_auth_controller::finish_social_sign_in),
        )
        .route(
            COMPLETE_SOCIAL_SIGNUP_ROUTE,
            post(social_auth_controller::complete_social_signup),
        )
        .route(
            CURRENT_ACCOUNT_ROUTE,
            get(account_session_controller::get_current_account),
        )
        .route(
            LOGOUT_ACCOUNT_ROUTE,
            post(account_session_controller::logout_account),
        )
}
