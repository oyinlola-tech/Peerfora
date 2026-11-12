mod jobs_loader;
mod middleware_loader;
mod routes_loader;

use crate::models::app_state::AppState;
use axum::Router;

pub fn register_loaders(state: AppState) -> Router {
    jobs_loader::start_background_jobs(state.database.clone());
    let app = routes_loader::register_routes(Router::new());
    middleware_loader::register_middlewares(app).with_state(state)
}
