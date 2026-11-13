use axum::{Json, extract::State};

use crate::{
    constants::health::{READINESS_STATUS, SERVICE_NAME},
    dtos::health_response::HealthResponse,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::health_service::{check_service_readiness, get_health_status},
};

pub async fn show_health() -> Json<HealthResponse> {
    Json(get_health_status())
}

pub async fn show_readiness(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, AppError> {
    check_service_readiness(&state.database).await?;
    Ok(Json(HealthResponse {
        status: READINESS_STATUS,
        service: SERVICE_NAME,
    }))
}
