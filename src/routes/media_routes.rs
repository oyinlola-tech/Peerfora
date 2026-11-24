use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{
    constants::phase_four::{
        MEDIA_DOWNLOAD_ROUTE, MEDIA_LIST_ROUTE, MEDIA_REPORT_ROUTE, MEDIA_ROUTE,
        MEDIA_UPLOAD_BYTES_ROUTE, MEDIA_UPLOAD_COMPLETE_ROUTE, MEDIA_UPLOADS_ROUTE,
    },
    controllers::media_controller,
    models::app_state::AppState,
};

pub fn build_routes() -> Router<AppState> {
    Router::new()
        .route(
            MEDIA_UPLOADS_ROUTE,
            post(media_controller::initiate_media_upload),
        )
        .route(
            MEDIA_UPLOAD_COMPLETE_ROUTE,
            put(media_controller::complete_media_upload),
        )
        .route(
            MEDIA_UPLOAD_BYTES_ROUTE,
            put(media_controller::upload_media_bytes),
        )
        .route(MEDIA_ROUTE, get(media_controller::get_media))
        .route(MEDIA_DOWNLOAD_ROUTE, get(media_controller::download_media))
        .route(MEDIA_LIST_ROUTE, get(media_controller::list_media))
        .route(MEDIA_REPORT_ROUTE, post(media_controller::report_media))
}
