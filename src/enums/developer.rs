use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperCredentialEnvironment {
    Sandbox,
    Production,
}

impl DeveloperCredentialEnvironment {
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
pub enum DeveloperCredentialValidation {
    Authorized(Uuid, Uuid),
    Invalid,
    ScopeDenied,
    RateLimited,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperApplicationType {
    Web,
    Server,
    AgentIntegration,
}

impl DeveloperApplicationType {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Server => "server",
            Self::AgentIntegration => "agent_integration",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "web" => Some(Self::Web),
            "server" => Some(Self::Server),
            "agent_integration" => Some(Self::AgentIntegration),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperApplicationScope {
    AcademicDirectoryRead,
}

impl DeveloperApplicationScope {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::AcademicDirectoryRead => "academic.directory.read",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "academic.directory.read" => Some(Self::AcademicDirectoryRead),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperApplicationStatus {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Suspended,
    Revoked,
}

impl DeveloperApplicationStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Submitted => "submitted",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "submitted" => Some(Self::Submitted),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "suspended" => Some(Self::Suspended),
            "revoked" => Some(Self::Revoked),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperApplicationReviewDecision {
    Approve,
    Reject,
}

impl DeveloperApplicationReviewDecision {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Approve => "approved",
            Self::Reject => "rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperApplicationScopeStatus {
    Requested,
    Approved,
    Rejected,
}

impl DeveloperApplicationScopeStatus {
    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "requested" => Some(Self::Requested),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperProfileStatus {
    Active,
    Suspended,
}

impl DeveloperProfileStatus {
    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "suspended" => Some(Self::Suspended),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DeveloperError {
    ProfileNameInvalid,
    ProfileDescriptionInvalid,
    ProfileAlreadyExists,
    ProfileUnavailable,
    ApplicationNameInvalid,
    ApplicationDescriptionInvalid,
    NoRequestedScopes,
    TooManyRequestedScopes,
    DuplicateRequestedScope,
    RequestedScopeUnavailable,
    ApplicationAlreadyExists,
    ApplicationNotFound,
    ApplicationCannotBeSubmitted,
    ApplicationNotAwaitingReview,
    ReviewReasonInvalid,
    CredentialInvalid,
    CredentialNameInvalid,
    CredentialLifetimeInvalid,
    ApplicationNotApproved,
    ApplicationSuspended,
    ApplicationStatusConflict,
    NoApprovedScopes,
    CredentialLimitReached,
    CredentialNotFound,
    CredentialNotActive,
    CredentialScopeDenied,
    CredentialEnvironmentMismatch,
    AdministrativeReasonInvalid,
}
