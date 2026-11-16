use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    constants::developers::{
        DEVELOPER_APPLICATION_CREDENTIAL_REVOKE_ROUTE,
        DEVELOPER_APPLICATION_CREDENTIAL_ROTATE_ROUTE, DEVELOPER_APPLICATION_CREDENTIALS_ROUTE,
        DEVELOPER_APPLICATION_SUBMIT_ROUTE, DEVELOPER_APPLICATIONS_ROUTE, DEVELOPER_PROFILE_ROUTE,
    },
    controllers::{developer_controller, developer_credential_controller},
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            DEVELOPER_PROFILE_ROUTE,
            post(developer_controller::activate_profile),
        )
        .route(
            DEVELOPER_APPLICATIONS_ROUTE,
            post(developer_controller::create_application),
        )
        .route(
            DEVELOPER_APPLICATIONS_ROUTE,
            get(developer_controller::list_my_applications),
        )
        .route(
            DEVELOPER_APPLICATION_SUBMIT_ROUTE,
            post(developer_controller::submit_application),
        )
        .route(
            DEVELOPER_APPLICATION_CREDENTIALS_ROUTE,
            post(developer_credential_controller::issue_credential)
                .get(developer_credential_controller::list_credentials),
        )
        .route(
            DEVELOPER_APPLICATION_CREDENTIAL_REVOKE_ROUTE,
            post(developer_credential_controller::revoke_own_credential),
        )
        .route(
            DEVELOPER_APPLICATION_CREDENTIAL_ROTATE_ROUTE,
            post(developer_credential_controller::rotate_credential),
        )
}
