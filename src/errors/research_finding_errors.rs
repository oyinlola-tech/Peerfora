use axum::http::StatusCode;

use crate::enums::research::ResearchFindingError;

pub fn get_research_finding_error_response(
    error: ResearchFindingError,
) -> (StatusCode, &'static str) {
    match error {
        ResearchFindingError::SourceTitleInvalid => (
            StatusCode::BAD_REQUEST,
            "Source title must contain between 1 and 500 characters",
        ),
        ResearchFindingError::SourceDoiInvalid => (
            StatusCode::BAD_REQUEST,
            "A DOI source must include a DOI between 1 and 255 characters",
        ),
        ResearchFindingError::SourceUrlInvalid => (
            StatusCode::BAD_REQUEST,
            "Source URL must be a valid HTTPS URL no longer than 2048 characters",
        ),
        ResearchFindingError::SourceAuthorsInvalid => (
            StatusCode::BAD_REQUEST,
            "A source can include at most 50 non-empty author names",
        ),
        ResearchFindingError::SourceYearInvalid => (
            StatusCode::BAD_REQUEST,
            "Source publication year must be between 1000 and 9999",
        ),
        ResearchFindingError::SourceProvenanceInvalid => (
            StatusCode::BAD_REQUEST,
            "Source provenance must contain between 1 and 2000 characters",
        ),
        ResearchFindingError::SourceLicenseInvalid => (
            StatusCode::BAD_REQUEST,
            "Source license cannot exceed 500 characters",
        ),
        ResearchFindingError::DuplicateDoi => (
            StatusCode::CONFLICT,
            "This DOI is already recorded in the research project",
        ),
        ResearchFindingError::SourceNotFound => {
            (StatusCode::NOT_FOUND, "Research source was not found")
        }
        ResearchFindingError::FindingTitleInvalid => (
            StatusCode::BAD_REQUEST,
            "Finding title must contain between 3 and 200 characters",
        ),
        ResearchFindingError::FindingStatementInvalid => (
            StatusCode::BAD_REQUEST,
            "Finding statement must contain between 1 and 10000 characters",
        ),
        ResearchFindingError::FindingDetailsInvalid => (
            StatusCode::BAD_REQUEST,
            "Finding details cannot exceed 20000 characters",
        ),
        ResearchFindingError::TooManyCitations => (
            StatusCode::BAD_REQUEST,
            "A finding revision can cite at most 50 sources",
        ),
        ResearchFindingError::DuplicateCitation => (
            StatusCode::BAD_REQUEST,
            "A source can only be cited once in a finding revision",
        ),
        ResearchFindingError::CitationNotInProject => (
            StatusCode::BAD_REQUEST,
            "Every cited source must belong to the research project",
        ),
        ResearchFindingError::CitationLocatorInvalid => (
            StatusCode::BAD_REQUEST,
            "Citation locator cannot exceed 500 characters",
        ),
        ResearchFindingError::CitationContextInvalid => (
            StatusCode::BAD_REQUEST,
            "Citation context cannot exceed 2000 characters",
        ),
        ResearchFindingError::ReviewNotesInvalid => (
            StatusCode::BAD_REQUEST,
            "Review notes cannot exceed 5000 characters",
        ),
        ResearchFindingError::ProjectNotFound => {
            (StatusCode::NOT_FOUND, "Research project was not found")
        }
        ResearchFindingError::FindingNotFound => {
            (StatusCode::NOT_FOUND, "Research finding was not found")
        }
        ResearchFindingError::PermissionDenied => (
            StatusCode::FORBIDDEN,
            "You cannot manage this research finding",
        ),
        ResearchFindingError::InvalidTransition => (
            StatusCode::CONFLICT,
            "This research finding cannot make that state change",
        ),
        ResearchFindingError::ReviewUnavailable => (
            StatusCode::CONFLICT,
            "This research finding has no revision awaiting review",
        ),
        ResearchFindingError::SelfReviewForbidden => (
            StatusCode::FORBIDDEN,
            "A finding contributor cannot review their own revision",
        ),
        ResearchFindingError::RevisionLimitReached => (
            StatusCode::CONFLICT,
            "This research finding has reached its revision limit",
        ),
    }
}
