use axum::http::StatusCode;

use crate::enums::discussion::DiscussionError;

pub fn get_discussion_error_response(error: DiscussionError) -> (StatusCode, &'static str) {
    match error {
        DiscussionError::TitleInvalid => (
            StatusCode::BAD_REQUEST,
            "Post title must contain between 3 and 200 characters",
        ),
        DiscussionError::BodyInvalid => (
            StatusCode::BAD_REQUEST,
            "Post body must contain between 1 and 20000 characters",
        ),
        DiscussionError::AccountCannotPost => (
            StatusCode::FORBIDDEN,
            "An active, email-verified account is required to post",
        ),
        DiscussionError::CommunityNotFound => (StatusCode::NOT_FOUND, "Community was not found"),
        DiscussionError::PostNotFound => (StatusCode::NOT_FOUND, "Post was not found"),
        DiscussionError::MembershipRequired => (
            StatusCode::FORBIDDEN,
            "An active community membership is required to post",
        ),
        DiscussionError::CommentBodyInvalid => (
            StatusCode::BAD_REQUEST,
            "Comment body must contain between 1 and 10000 characters",
        ),
        DiscussionError::ParentCommentNotFound => {
            (StatusCode::NOT_FOUND, "Parent comment was not found")
        }
        DiscussionError::PostLocked => (StatusCode::CONFLICT, "This discussion is locked"),
    }
}
