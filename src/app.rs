use axum::Router;

use crate::{loaders::register_loaders, models::app_state::AppState};

pub fn build_app(state: AppState) -> Router {
    register_loaders(state)
}
