use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchProjectVisibility {
    #[default]
    Private,
    InvitationOnly,
    Public,
}

impl ResearchProjectVisibility {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::InvitationOnly => "invitation_only",
            Self::Public => "public",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchProjectMemberRole {
    Supervisor,
    Researcher,
    Contributor,
    Reviewer,
    Observer,
}

impl ResearchProjectMemberRole {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Supervisor => "supervisor",
            Self::Researcher => "researcher",
            Self::Contributor => "contributor",
            Self::Reviewer => "reviewer",
            Self::Observer => "observer",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchProjectError {
    TitleInvalid,
    AbstractTooLong,
    DescriptionTooLong,
    TooManyObjectives,
    InvalidObjective,
    TooManyQuestions,
    InvalidQuestion,
    TooManySubjects,
    DuplicateSubject,
    SubjectUnavailable,
    AccountUnavailable,
    ProjectNotFound,
    ProjectUnavailable,
    MemberNotFound,
    InvitationNotFound,
    InvitationAlreadyExists,
    InviteeUnavailable,
    OwnerCannotLeave,
    CannotRemoveProjectOwner,
    PermissionDenied,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchTaskStatus {
    Draft,
    Queued,
    Assigned,
    Running,
    AwaitingReview,
    Completed,
    Failed,
    Cancelled,
}

impl ResearchTaskStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Queued => "queued",
            Self::Assigned => "assigned",
            Self::Running => "running",
            Self::AwaitingReview => "awaiting_review",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchTaskReviewDecision {
    Approved,
    RevisionRequested,
    Rejected,
}

impl ResearchTaskReviewDecision {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::RevisionRequested => "revision_requested",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchTaskError {
    TitleInvalid,
    ObjectiveInvalid,
    ScopeInvalid,
    TooManyDataSources,
    InvalidDataSource,
    TooManyPermittedTools,
    InvalidPermittedTool,
    MissingDeliverables,
    TooManyDeliverables,
    InvalidDeliverable,
    AssigneeUsernameInvalid,
    RuntimeLimitInvalid,
    ResultLimitInvalid,
    ResultInvalid,
    ReviewNotesTooLong,
    ProjectNotFound,
    TaskNotFound,
    PermissionDenied,
    InvalidTransition,
    AssigneeUnavailable,
    AssignmentUnavailable,
    SubmissionUnavailable,
    SubmissionLimitReached,
    SelfReviewForbidden,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchSourceType {
    Publication,
    Doi,
    ExternalUrl,
    Dataset,
}

impl ResearchSourceType {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Publication => "publication",
            Self::Doi => "doi",
            Self::ExternalUrl => "external_url",
            Self::Dataset => "dataset",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchFindingReviewDecision {
    Approve,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchFindingStatus {
    Draft,
    InReview,
    Approved,
    Published,
    CorrectionDraft,
    CorrectionInReview,
    CorrectionApproved,
    Retracted,
}

impl ResearchFindingStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Published => "published",
            Self::CorrectionDraft => "correction_draft",
            Self::CorrectionInReview => "correction_in_review",
            Self::CorrectionApproved => "correction_approved",
            Self::Retracted => "retracted",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "in_review" => Some(Self::InReview),
            "approved" => Some(Self::Approved),
            "published" => Some(Self::Published),
            "correction_draft" => Some(Self::CorrectionDraft),
            "correction_in_review" => Some(Self::CorrectionInReview),
            "correction_approved" => Some(Self::CorrectionApproved),
            "retracted" => Some(Self::Retracted),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchFindingRevisionStatus {
    Draft,
    InReview,
    Approved,
    Rejected,
    Published,
    Superseded,
}

impl ResearchFindingRevisionStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Published => "published",
            Self::Superseded => "superseded",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "in_review" => Some(Self::InReview),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "published" => Some(Self::Published),
            "superseded" => Some(Self::Superseded),
            _ => None,
        }
    }
}

impl ResearchFindingReviewDecision {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Approve => "approved",
            Self::Reject => "rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchFindingError {
    SourceTitleInvalid,
    SourceDoiInvalid,
    SourceUrlInvalid,
    SourceAuthorsInvalid,
    SourceYearInvalid,
    SourceProvenanceInvalid,
    SourceLicenseInvalid,
    DuplicateDoi,
    SourceNotFound,
    FindingTitleInvalid,
    FindingStatementInvalid,
    FindingDetailsInvalid,
    TooManyCitations,
    DuplicateCitation,
    CitationNotInProject,
    CitationLocatorInvalid,
    CitationContextInvalid,
    ReviewNotesInvalid,
    ProjectNotFound,
    FindingNotFound,
    PermissionDenied,
    InvalidTransition,
    ReviewUnavailable,
    SelfReviewForbidden,
    RevisionLimitReached,
}
