use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{
    constants::agents::{
        AGENT_API_RESEARCH_TASK_ACCEPT_ROUTE, AGENT_API_RESEARCH_TASK_ROUTE,
        AGENT_API_RESEARCH_TASK_START_ROUTE, AGENT_API_RESEARCH_TASK_SUBMISSIONS_ROUTE,
        AGENT_API_RESEARCH_TASKS_ROUTE, AGENT_API_SUBJECTS_ROUTE,
    },
    controllers::{agent_api_controller, agent_research_task_controller},
    enums::agent::AgentCapability,
    middlewares::agent_credential_authentication::{
        agent_credential_guard, require_agent_credential,
    },
    models::app_state::AppState,
};

pub fn build_routes(state: AppState) -> Router<AppState> {
    let public_content_routes = Router::new()
        .route(
            AGENT_API_SUBJECTS_ROUTE,
            get(agent_api_controller::search_subjects),
        )
        .route_layer(from_fn_with_state(
            agent_credential_guard(&state, AgentCapability::ReadPublicContent),
            require_agent_credential,
        ));
    let research_routes = Router::new()
        .route(
            AGENT_API_RESEARCH_TASKS_ROUTE,
            get(agent_research_task_controller::list_assigned_research_tasks),
        )
        .route(
            AGENT_API_RESEARCH_TASK_ROUTE,
            get(agent_research_task_controller::find_assigned_research_task),
        )
        .route(
            AGENT_API_RESEARCH_TASK_ACCEPT_ROUTE,
            post(agent_research_task_controller::accept_research_task),
        )
        .route(
            AGENT_API_RESEARCH_TASK_START_ROUTE,
            post(agent_research_task_controller::start_research_task),
        )
        .route(
            AGENT_API_RESEARCH_TASK_SUBMISSIONS_ROUTE,
            post(agent_research_task_controller::submit_research_task_result),
        )
        .route_layer(from_fn_with_state(
            agent_credential_guard(&state, AgentCapability::Research),
            require_agent_credential,
        ));
    Router::new()
        .merge(public_content_routes)
        .merge(research_routes)
}
