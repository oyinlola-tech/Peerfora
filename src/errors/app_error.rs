use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Internal,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred",
            ),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}
