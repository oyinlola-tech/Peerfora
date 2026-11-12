use axum::Json;

use crate::{dtos::health_response::HealthResponse, services::health_service::get_health_status};

pub async fn show_health() -> Json<HealthResponse> {
    Json(get_health_status())
}
