use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    constants::agents::{
        AGENT_CLAIM_COMPLETION_ROUTE, AGENT_CREDENTIAL_REVOKE_ROUTE, AGENT_CREDENTIAL_ROTATE_ROUTE,
        AGENT_CREDENTIALS_ROUTE, AGENT_PUBLIC_PROFILE_ROUTE, AGENT_REGISTRATIONS_ROUTE,
        AGENTS_ROUTE, MY_AGENTS_ROUTE,
    },
    controllers::{agent_credential_controller, agent_identity_controller},
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            AGENT_PUBLIC_PROFILE_ROUTE,
            get(agent_identity_controller::find_public_agent_profile),
        )
        .route(
            AGENT_REGISTRATIONS_ROUTE,
            post(agent_identity_controller::initiate_agent_registration),
        )
        .route(
            AGENT_CLAIM_COMPLETION_ROUTE,
            post(agent_identity_controller::complete_agent_operator_claim),
        )
        .route(
            AGENTS_ROUTE,
            post(agent_identity_controller::register_agent),
        )
        .route(
            MY_AGENTS_ROUTE,
            get(agent_identity_controller::list_my_agents),
        )
        .route(
            AGENT_CREDENTIALS_ROUTE,
            post(agent_credential_controller::issue_credential)
                .get(agent_credential_controller::list_credentials),
        )
        .route(
            AGENT_CREDENTIAL_ROTATE_ROUTE,
            post(agent_credential_controller::rotate_credential),
        )
        .route(
            AGENT_CREDENTIAL_REVOKE_ROUTE,
            post(agent_credential_controller::revoke_credential),
        )
}
