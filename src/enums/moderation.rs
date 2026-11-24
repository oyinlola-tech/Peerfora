use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationReportCategory {
    Spam,
    Harassment,
    Bullying,
    HateAndDiscrimination,
    Threats,
    Doxxing,
    SexualExploitation,
    Impersonation,
    Scams,
    MaliciousLinks,
    AcademicCheating,
    Copyright,
    IllegalContent,
    CoordinatedAbuse,
    Other,
}

impl ModerationReportCategory {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Spam => "spam",
            Self::Harassment => "harassment",
            Self::Bullying => "bullying",
            Self::HateAndDiscrimination => "hate_and_discrimination",
            Self::Threats => "threats",
            Self::Doxxing => "doxxing",
            Self::SexualExploitation => "sexual_exploitation",
            Self::Impersonation => "impersonation",
            Self::Scams => "scams",
            Self::MaliciousLinks => "malicious_links",
            Self::AcademicCheating => "academic_cheating",
            Self::Copyright => "copyright",
            Self::IllegalContent => "illegal_content",
            Self::CoordinatedAbuse => "coordinated_abuse",
            Self::Other => "other",
        }
    }

    pub fn from_category(value: &str) -> Result<Self, ModerationError> {
        match value {
            "spam" => Ok(Self::Spam),
            "harassment" => Ok(Self::Harassment),
            "bullying" => Ok(Self::Bullying),
            "hate_and_discrimination" => Ok(Self::HateAndDiscrimination),
            "threats" => Ok(Self::Threats),
            "doxxing" => Ok(Self::Doxxing),
            "sexual_exploitation" => Ok(Self::SexualExploitation),
            "impersonation" => Ok(Self::Impersonation),
            "scams" => Ok(Self::Scams),
            "malicious_links" => Ok(Self::MaliciousLinks),
            "academic_cheating" => Ok(Self::AcademicCheating),
            "copyright" => Ok(Self::Copyright),
            "illegal_content" => Ok(Self::IllegalContent),
            "coordinated_abuse" => Ok(Self::CoordinatedAbuse),
            "other" => Ok(Self::Other),
            _ => Err(ModerationError::DescriptionInvalid),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationReportDecision {
    Resolve,
    Dismiss,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationReportStatus {
    Open,
    Assigned,
    Resolved,
    Dismissed,
}

impl ModerationReportStatus {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Assigned => "assigned",
            Self::Resolved => "resolved",
            Self::Dismissed => "dismissed",
        }
    }
}

impl ModerationReportDecision {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Resolve => "resolved",
            Self::Dismiss => "dismissed",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ModerationError {
    DescriptionInvalid,
    ResolutionReasonInvalid,
    ReportTargetUnavailable,
    DuplicateActiveReport,
    ReportNotFound,
    ReportAlreadyClosed,
    AdministratorUnavailable,
}
