use axum::Router;

use crate::routes;

pub fn register_routes(app: Router) -> Router {
    app.merge(routes::health_routes::build_routes())
}
