use axum::Router;

use crate::{models::app_state::AppState, routes};

pub fn register_routes(app: Router<AppState>) -> Router<AppState> {
    app.merge(routes::health_routes::build_routes())
        .merge(routes::account_auth_routes::build_routes())
}
