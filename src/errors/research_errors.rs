use axum::http::StatusCode;

use crate::enums::research::{ResearchProjectError, ResearchTaskError};

pub fn get_research_project_error_response(
    error: ResearchProjectError,
) -> (StatusCode, &'static str) {
    match error {
        ResearchProjectError::TitleInvalid => (
            StatusCode::BAD_REQUEST,
            "Project title must contain between 3 and 160 characters",
        ),
        ResearchProjectError::AbstractTooLong => (
            StatusCode::BAD_REQUEST,
            "Project abstract cannot exceed 2000 characters",
        ),
        ResearchProjectError::DescriptionTooLong => (
            StatusCode::BAD_REQUEST,
            "Project description cannot exceed 10000 characters",
        ),
        ResearchProjectError::TooManyObjectives => (
            StatusCode::BAD_REQUEST,
            "A project can have at most 20 objectives",
        ),
        ResearchProjectError::InvalidObjective => (
            StatusCode::BAD_REQUEST,
            "Each research objective must contain between 1 and 500 characters",
        ),
        ResearchProjectError::TooManyQuestions => (
            StatusCode::BAD_REQUEST,
            "A project can have at most 20 research questions",
        ),
        ResearchProjectError::InvalidQuestion => (
            StatusCode::BAD_REQUEST,
            "Each research question must contain between 1 and 500 characters",
        ),
        ResearchProjectError::TooManySubjects => (
            StatusCode::BAD_REQUEST,
            "A project can be linked to at most 20 subjects",
        ),
        ResearchProjectError::DuplicateSubject => (
            StatusCode::BAD_REQUEST,
            "A subject can only be linked to a project once",
        ),
        ResearchProjectError::SubjectUnavailable => (
            StatusCode::BAD_REQUEST,
            "One or more selected subjects are unavailable",
        ),
        ResearchProjectError::AccountUnavailable => (
            StatusCode::FORBIDDEN,
            "This account cannot create a research project",
        ),
        ResearchProjectError::ProjectNotFound => {
            (StatusCode::NOT_FOUND, "Research project was not found")
        }
        ResearchProjectError::InvitationNotFound => (
            StatusCode::NOT_FOUND,
            "Research project invitation was not found",
        ),
        ResearchProjectError::MemberNotFound => (
            StatusCode::NOT_FOUND,
            "Research project member was not found",
        ),
        ResearchProjectError::ProjectUnavailable => (
            StatusCode::CONFLICT,
            "This research project is not accepting changes",
        ),
        ResearchProjectError::InvitationAlreadyExists => (
            StatusCode::CONFLICT,
            "This person already has access or a pending invitation",
        ),
        ResearchProjectError::InviteeUnavailable => (
            StatusCode::NOT_FOUND,
            "The invited account is not available",
        ),
        ResearchProjectError::OwnerCannotLeave => (
            StatusCode::CONFLICT,
            "Transfer project ownership before leaving",
        ),
        ResearchProjectError::CannotRemoveProjectOwner => (
            StatusCode::CONFLICT,
            "Transfer project ownership before removing its owner",
        ),
        ResearchProjectError::PermissionDenied => (
            StatusCode::FORBIDDEN,
            "You cannot manage this research project",
        ),
    }
}

pub fn get_research_task_error_response(error: ResearchTaskError) -> (StatusCode, &'static str) {
    match error {
        ResearchTaskError::TitleInvalid => (
            StatusCode::BAD_REQUEST,
            "Task title must contain between 3 and 160 characters",
        ),
        ResearchTaskError::ObjectiveInvalid => (
            StatusCode::BAD_REQUEST,
            "Task objective must contain between 1 and 3000 characters",
        ),
        ResearchTaskError::ScopeInvalid => (
            StatusCode::BAD_REQUEST,
            "Task scope must contain between 1 and 5000 characters",
        ),
        ResearchTaskError::TooManyDataSources => (
            StatusCode::BAD_REQUEST,
            "A research task can reference at most 20 data sources",
        ),
        ResearchTaskError::InvalidDataSource => (
            StatusCode::BAD_REQUEST,
            "Each data source reference must contain between 1 and 500 characters",
        ),
        ResearchTaskError::TooManyPermittedTools => (
            StatusCode::BAD_REQUEST,
            "A research task can permit at most 20 tools",
        ),
        ResearchTaskError::InvalidPermittedTool => (
            StatusCode::BAD_REQUEST,
            "Each permitted tool name must contain between 1 and 100 characters",
        ),
        ResearchTaskError::MissingDeliverables => (
            StatusCode::BAD_REQUEST,
            "A research task requires at least one expected deliverable",
        ),
        ResearchTaskError::TooManyDeliverables => (
            StatusCode::BAD_REQUEST,
            "A research task can define at most 20 expected deliverables",
        ),
        ResearchTaskError::InvalidDeliverable => (
            StatusCode::BAD_REQUEST,
            "Each expected deliverable must contain between 1 and 500 characters",
        ),
        ResearchTaskError::AssigneeUsernameInvalid => (
            StatusCode::BAD_REQUEST,
            "Username must be 3 to 24 letters, numbers, or underscores",
        ),
        ResearchTaskError::RuntimeLimitInvalid => (
            StatusCode::BAD_REQUEST,
            "Task runtime limit must be between 1 and 86400 seconds",
        ),
        ResearchTaskError::ResultLimitInvalid => (
            StatusCode::BAD_REQUEST,
            "Task result limit must be between 1 and 1000000 bytes",
        ),
        ResearchTaskError::ResultInvalid => (
            StatusCode::BAD_REQUEST,
            "Task result must be non-empty and within the task result limit",
        ),
        ResearchTaskError::ReviewNotesTooLong => (
            StatusCode::BAD_REQUEST,
            "Task review notes cannot exceed 5000 bytes",
        ),
        ResearchTaskError::ProjectNotFound => {
            (StatusCode::NOT_FOUND, "Research project was not found")
        }
        ResearchTaskError::TaskNotFound => (StatusCode::NOT_FOUND, "Research task was not found"),
        ResearchTaskError::PermissionDenied => (
            StatusCode::FORBIDDEN,
            "You cannot manage this research task",
        ),
        ResearchTaskError::InvalidTransition => (
            StatusCode::CONFLICT,
            "This research task cannot make that state change",
        ),
        ResearchTaskError::AssigneeUnavailable => (
            StatusCode::NOT_FOUND,
            "The selected assignee is not an active project member",
        ),
        ResearchTaskError::AssignmentUnavailable => (
            StatusCode::CONFLICT,
            "This research task is not assigned to you",
        ),
        ResearchTaskError::SubmissionUnavailable => (
            StatusCode::CONFLICT,
            "This research task has no submission awaiting review",
        ),
        ResearchTaskError::SubmissionLimitReached => (
            StatusCode::CONFLICT,
            "This research task has reached its submission limit",
        ),
        ResearchTaskError::SelfReviewForbidden => (
            StatusCode::FORBIDDEN,
            "A research task assignee cannot review their own result",
        ),
    }
}
