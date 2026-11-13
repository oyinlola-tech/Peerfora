use crate::{
    constants::health::{HEALTH_STATUS, SERVICE_NAME},
    dtos::health_response::HealthResponse,
    errors::app_error::AppError,
    repositories::database_repository,
};
use sqlx::PgPool;

pub fn get_health_status() -> HealthResponse {
    HealthResponse {
        status: HEALTH_STATUS,
        service: SERVICE_NAME,
    }
}

pub async fn check_service_readiness(database: &PgPool) -> Result<(), AppError> {
    database_repository::check_database_readiness(database).await
}
