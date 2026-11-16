use axum::{Router, routing::post};

use crate::{
    constants::moderation::{
        ACCOUNT_REPORT_ROUTE, COMMENT_REPORT_ROUTE, COMMUNITY_REPORT_ROUTE, POST_REPORT_ROUTE,
        RESEARCH_FINDING_REPORT_ROUTE, RESEARCH_PROJECT_REPORT_ROUTE,
    },
    controllers::moderation_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            ACCOUNT_REPORT_ROUTE,
            post(moderation_controller::report_account),
        )
        .route(
            COMMUNITY_REPORT_ROUTE,
            post(moderation_controller::report_community),
        )
        .route(POST_REPORT_ROUTE, post(moderation_controller::report_post))
        .route(
            COMMENT_REPORT_ROUTE,
            post(moderation_controller::report_comment),
        )
        .route(
            RESEARCH_PROJECT_REPORT_ROUTE,
            post(moderation_controller::report_research_project),
        )
        .route(
            RESEARCH_FINDING_REPORT_ROUTE,
            post(moderation_controller::report_research_finding),
        )
}
