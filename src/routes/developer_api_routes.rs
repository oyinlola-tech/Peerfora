use axum::{Router, middleware::from_fn_with_state, routing::get};

use crate::{
    constants::developers::{
        DEVELOPER_API_SUBJECTS_ROUTE, DEVELOPER_SCOPE_ACADEMIC_DIRECTORY_READ,
    },
    controllers::developer_api_controller,
    middlewares::developer_credential_authentication::{
        developer_credential_guard, require_developer_credential,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            DEVELOPER_API_SUBJECTS_ROUTE,
            get(developer_api_controller::search_subjects),
        )
        .route_layer(from_fn_with_state(
            developer_credential_guard(&state, DEVELOPER_SCOPE_ACADEMIC_DIRECTORY_READ),
            require_developer_credential,
        ))
}
