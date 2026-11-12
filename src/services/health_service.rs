use crate::{
    constants::health::{HEALTH_STATUS, SERVICE_NAME},
    dtos::health_response::HealthResponse,
};

pub fn get_health_status() -> HealthResponse {
    HealthResponse {
        status: HEALTH_STATUS,
        service: SERVICE_NAME,
    }
}
