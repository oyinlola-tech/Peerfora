use axum::Router;

use crate::{models::app_state::AppState, routes};

pub fn register_routes(app: Router<AppState>, state: AppState) -> Router<AppState> {
    app.merge(routes::health_routes::build_routes())
        .merge(routes::admin_auth_routes::build_routes(state.clone()))
        .merge(routes::admin_account_routes::build_routes(state.clone()))
        .merge(routes::admin_administrator_routes::build_routes(
            state.clone(),
        ))
        .merge(routes::admin_community_routes::build_routes(state.clone()))
        .merge(routes::admin_communication_routes::build_routes(
            state.clone(),
        ))
        .merge(routes::admin_moderation_routes::build_routes(state.clone()))
        .merge(routes::admin_developer_routes::build_routes(state.clone()))
        .merge(routes::admin_agent_routes::build_routes(state.clone()))
        .merge(routes::admin_research_routes::build_routes(state.clone()))
        .merge(routes::admin_knowledge_routes::build_routes(state.clone()))
        .merge(routes::admin_role_routes::build_routes(state.clone()))
        .merge(routes::account_auth_routes::build_routes())
        .merge(routes::agent_routes::build_routes())
        .merge(routes::agent_api_routes::build_routes(state.clone()))
        .merge(routes::academic_directory_routes::build_routes())
        .merge(routes::institution_directory_routes::build_routes())
        .merge(routes::community_routes::build_routes())
        .merge(routes::discussion_routes::build_routes())
        .merge(routes::developer_routes::build_routes())
        .merge(routes::developer_api_routes::build_routes(state.clone()))
        .merge(routes::moderation_routes::build_routes())
        .merge(routes::communication_routes::build_routes())
        .merge(routes::media_routes::build_routes())
        .merge(routes::realtime_routes::build_routes())
        .merge(routes::profile_routes::build_routes())
        .merge(routes::research_routes::build_routes())
        .merge(routes::knowledge_routes::build_routes())
}
