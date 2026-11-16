use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCredentialEnvironment {
    Sandbox,
    Production,
}

impl AgentCredentialEnvironment {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Sandbox => "sandbox",
            Self::Production => "production",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "sandbox" => Some(Self::Sandbox),
            "production" => Some(Self::Production),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AgentCredentialValidation {
    Authorized(Uuid, Uuid),
    Invalid,
    ScopeDenied,
    RateLimited,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentReviewDecision {
    Approve,
    Reject,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentLifecycleAction {
    Suspend,
    Reinstate,
    Revoke,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentOperatorVerificationStatus {
    EmailClaimedByActiveVerifiedAccount,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstitutionDirectoryVerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
    Inactive,
}

impl InstitutionDirectoryVerificationStatus {
    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "unverified" => Some(Self::Unverified),
            "pending" => Some(Self::Pending),
            "verified" => Some(Self::Verified),
            "rejected" => Some(Self::Rejected),
            "inactive" => Some(Self::Inactive),
            _ => None,
        }
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentCapabilityReviewStatus {
    Requested,
    Approved,
    Denied,
    Revoked,
}

impl AgentCapabilityReviewStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Approved => "approved",
            Self::Denied => "denied",
            Self::Revoked => "revoked",
        }
    }
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
    PendingClaim,
    PendingReview,
    Approved,
    Active,
    Restricted,
    Paused,
    Rejected,
    Suspended,
    Revoked,
    Deleted,
}

impl AgentReviewStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::PendingClaim => "pending_claim",
            Self::PendingReview => "pending_review",
            Self::Approved => "approved",
            Self::Active => "active",
            Self::Restricted => "restricted",
            Self::Paused => "paused",
            Self::Rejected => "rejected",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
            Self::Deleted => "deleted",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "pending_claim" => Some(Self::PendingClaim),
            "pending_review" => Some(Self::PendingReview),
            "approved" => Some(Self::Approved),
            "active" => Some(Self::Active),
            "restricted" => Some(Self::Restricted),
            "paused" => Some(Self::Paused),
            "rejected" => Some(Self::Rejected),
            "suspended" => Some(Self::Suspended),
            "revoked" => Some(Self::Revoked),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }
}

impl AgentReviewDecision {
    pub fn resulting_status(self) -> AgentReviewStatus {
        match self {
            Self::Approve => AgentReviewStatus::Approved,
            Self::Reject => AgentReviewStatus::Rejected,
        }
    }
}
