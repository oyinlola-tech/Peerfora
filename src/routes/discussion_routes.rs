use axum::{Router, routing::get};

use crate::{
    constants::discussions::{COMMUNITY_POSTS_ROUTE, POST_COMMENTS_ROUTE},
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
}
