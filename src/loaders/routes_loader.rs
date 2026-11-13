use axum::Router;

use crate::{models::app_state::AppState, routes};

pub fn register_routes(app: Router<AppState>, state: AppState) -> Router<AppState> {
    app.merge(routes::health_routes::build_routes())
        .merge(routes::admin_auth_routes::build_routes(state))
        .merge(routes::account_auth_routes::build_routes())
        .merge(routes::academic_directory_routes::build_routes())
        .merge(routes::community_routes::build_routes())
        .merge(routes::profile_routes::build_routes())
}
