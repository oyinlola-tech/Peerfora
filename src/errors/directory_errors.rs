use axum::http::StatusCode;

use crate::enums::directory_error::DirectoryError;

pub fn get_directory_error_response(error: DirectoryError) -> (StatusCode, &'static str) {
    match error {
        DirectoryError::SearchTermTooLong => (
            StatusCode::BAD_REQUEST,
            "Search term cannot exceed 100 characters",
        ),
    }
}
