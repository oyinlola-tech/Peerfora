use axum::http::StatusCode;

use crate::enums::community_error::CommunityError;

pub fn get_community_error_response(error: CommunityError) -> (StatusCode, &'static str) {
    match error {
        CommunityError::NameInvalid => (
            StatusCode::BAD_REQUEST,
            "Community name must contain between 3 and 80 characters",
        ),
        CommunityError::SlugInvalid => (
            StatusCode::BAD_REQUEST,
            "Community slug must use letters, numbers, and single hyphens",
        ),
        CommunityError::LanguageTagInvalid => {
            (StatusCode::BAD_REQUEST, "Language tag is not valid")
        }
        CommunityError::DescriptionTooLong => (
            StatusCode::BAD_REQUEST,
            "Community description cannot exceed 2000 characters",
        ),
        CommunityError::InvalidCategoryReferences => (
            StatusCode::BAD_REQUEST,
            "The selected community category needs a matching academic directory record",
        ),
        CommunityError::DuplicateSubject => (
            StatusCode::BAD_REQUEST,
            "A subject can only be linked to a community once",
        ),
        CommunityError::SubjectListTooLarge => (
            StatusCode::BAD_REQUEST,
            "A community can be linked to at most 20 subjects",
        ),
        CommunityError::SubjectUnavailable => (
            StatusCode::BAD_REQUEST,
            "One or more selected subjects are unavailable",
        ),
        CommunityError::InstitutionUnavailable => (
            StatusCode::BAD_REQUEST,
            "The selected institution is unavailable",
        ),
        CommunityError::CourseUnavailable => (
            StatusCode::BAD_REQUEST,
            "The selected course is unavailable",
        ),
        CommunityError::CommunityNotFound => (StatusCode::NOT_FOUND, "Community was not found"),
        CommunityError::CommunitySlugAlreadyUsed => (
            StatusCode::CONFLICT,
            "That community name is already in use",
        ),
        CommunityError::MembershipNotAllowed => {
            (StatusCode::FORBIDDEN, "You cannot join this community")
        }
        CommunityError::MembershipBanned => {
            (StatusCode::FORBIDDEN, "You cannot join this community")
        }
        CommunityError::OwnerCannotLeave => (
            StatusCode::CONFLICT,
            "Transfer community ownership before leaving",
        ),
    }
}
