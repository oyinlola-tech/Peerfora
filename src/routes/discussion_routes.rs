use axum::{Router, routing::get};

use crate::{
    constants::discussions::{
        COMMUNITY_POSTS_ROUTE, POST_COMMENTS_ROUTE, POST_RESEARCH_FINDING_REFERENCE_ROUTE,
        POST_RESEARCH_FINDING_REFERENCES_ROUTE,
    },
    controllers::discussion_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            COMMUNITY_POSTS_ROUTE,
            get(discussion_controller::list_community_posts)
                .post(discussion_controller::create_community_post),
        )
        .route(
            POST_COMMENTS_ROUTE,
            get(discussion_controller::list_post_comments)
                .post(discussion_controller::create_post_comment),
        )
        .route(
            POST_RESEARCH_FINDING_REFERENCES_ROUTE,
            get(discussion_controller::list_post_research_finding_references)
                .post(discussion_controller::link_public_research_finding_to_post),
        )
        .route(
            POST_RESEARCH_FINDING_REFERENCE_ROUTE,
            axum::routing::delete(discussion_controller::remove_post_research_finding_reference),
        )
}
