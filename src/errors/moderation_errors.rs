use axum::http::StatusCode;

use crate::enums::moderation::ModerationError;

pub fn get_moderation_error_response(error: ModerationError) -> (StatusCode, &'static str) {
    match error {
        ModerationError::DescriptionInvalid => (
            StatusCode::BAD_REQUEST,
            "Report description must contain between 1 and 2000 characters",
        ),
        ModerationError::ResolutionReasonInvalid => (
            StatusCode::BAD_REQUEST,
            "Resolution reason must contain between 1 and 1000 characters",
        ),
        ModerationError::ReportTargetUnavailable => {
            (StatusCode::NOT_FOUND, "The report target is unavailable")
        }
        ModerationError::DuplicateActiveReport => (
            StatusCode::CONFLICT,
            "An active report for this category and target already exists",
        ),
        ModerationError::ReportNotFound => (StatusCode::NOT_FOUND, "Report was not found"),
        ModerationError::ReportAlreadyClosed => {
            (StatusCode::CONFLICT, "This report has already been closed")
        }
        ModerationError::AdministratorUnavailable => (
            StatusCode::NOT_FOUND,
            "An active report reviewer with the required permission was not found",
        ),
    }
}
