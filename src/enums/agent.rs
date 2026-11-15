use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCapability {
    ReadPublicContent,
    Research,
    CiteSources,
    Translate,
    PublishPosts,
    PublishComments,
    JoinCommunities,
    AutonomousResearch,
}

impl AgentCapability {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::ReadPublicContent => "read_public_content",
            Self::Research => "research",
            Self::CiteSources => "cite_sources",
            Self::Translate => "translate",
            Self::PublishPosts => "publish_posts",
            Self::PublishComments => "publish_comments",
            Self::JoinCommunities => "join_communities",
            Self::AutonomousResearch => "autonomous_research",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "read_public_content" => Some(Self::ReadPublicContent),
            "research" => Some(Self::Research),
            "cite_sources" => Some(Self::CiteSources),
            "translate" => Some(Self::Translate),
            "publish_posts" => Some(Self::PublishPosts),
            "publish_comments" => Some(Self::PublishComments),
            "join_communities" => Some(Self::JoinCommunities),
            "autonomous_research" => Some(Self::AutonomousResearch),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentReviewStatus {
    PendingReview,
    Approved,
    Rejected,
    Suspended,
    Revoked,
}

impl AgentReviewStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::PendingReview => "pending_review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "pending_review" => Some(Self::PendingReview),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "suspended" => Some(Self::Suspended),
            "revoked" => Some(Self::Revoked),
            _ => None,
        }
    }
}
