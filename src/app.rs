use axum::Router;

use crate::loaders::register_loaders;

pub fn build_app() -> Router {
    register_loaders()
}
