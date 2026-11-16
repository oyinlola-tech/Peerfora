use axum::http::StatusCode;

use crate::enums::knowledge_error::KnowledgeError;

pub fn get_knowledge_error_response(error: KnowledgeError) -> (StatusCode, &'static str) {
    match error {
        KnowledgeError::ProposalInvalid => (
            StatusCode::BAD_REQUEST,
            "The knowledge proposal does not meet the supported requirements",
        ),
        KnowledgeError::CitationInvalid => (
            StatusCode::BAD_REQUEST,
            "Each citation must include valid bibliographic details and a DOI or HTTPS URL",
        ),
        KnowledgeError::SubjectUnavailable => (
            StatusCode::BAD_REQUEST,
            "The selected academic subject is unavailable",
        ),
        KnowledgeError::ResearchFindingUnavailable => (
            StatusCode::BAD_REQUEST,
            "A linked research finding is not publicly available",
        ),
        KnowledgeError::ProposalNotFound => (
            StatusCode::NOT_FOUND,
            "The knowledge proposal was not found",
        ),
        KnowledgeError::ProposalNotAwaitingReview => (
            StatusCode::CONFLICT,
            "The knowledge proposal is no longer awaiting review",
        ),
        KnowledgeError::EntryNotApproved => (
            StatusCode::CONFLICT,
            "Only an approved knowledge entry can be published",
        ),
        KnowledgeError::EntryNotPublishable => (
            StatusCode::CONFLICT,
            "The knowledge entry cannot be published in its current state",
        ),
        KnowledgeError::ReviewReasonInvalid => (
            StatusCode::BAD_REQUEST,
            "The review reason must contain between 1 and 1000 characters",
        ),
    }
}
