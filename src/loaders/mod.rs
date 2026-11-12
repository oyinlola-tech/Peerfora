mod middleware_loader;
mod routes_loader;

use axum::Router;

pub fn register_loaders() -> Router {
    let app = routes_loader::register_routes(Router::new());
    middleware_loader::register_middlewares(app)
}
