use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunityPostKind {
    Text,
    Question,
    Discussion,
}

impl CommunityPostKind {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Question => "question",
            Self::Discussion => "discussion",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DiscussionError {
    TitleInvalid,
    BodyInvalid,
    AccountCannotPost,
    CommunityNotFound,
    PostNotFound,
    MembershipRequired,
    CommentBodyInvalid,
    ParentCommentNotFound,
    PostLocked,
    ResearchFindingUnavailable,
    ResearchFindingAlreadyLinked,
    ResearchFindingReferenceLimitReached,
    PostResearchReferencePermissionDenied,
}
