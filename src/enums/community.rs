#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunityCategory {
    #[default]
    AcademicSubject,
    Institution,
    InstitutionCourse,
    AcademicDiscipline,
    StudyGroup,
    Scholarship,
    Internship,
    Career,
    StudentLife,
    General,
    Other,
}

impl CommunityCategory {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::AcademicSubject => "academic_subject",
            Self::Institution => "institution",
            Self::InstitutionCourse => "institution_course",
            Self::AcademicDiscipline => "academic_discipline",
            Self::StudyGroup => "study_group",
            Self::Scholarship => "scholarship",
            Self::Internship => "internship",
            Self::Career => "career",
            Self::StudentLife => "student_life",
            Self::General => "general",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for CommunityCategory {
    type Error = crate::enums::admin_auth_error::AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "academic_subject" => Ok(Self::AcademicSubject),
            "institution" => Ok(Self::Institution),
            "institution_course" => Ok(Self::InstitutionCourse),
            "academic_discipline" => Ok(Self::AcademicDiscipline),
            "study_group" => Ok(Self::StudyGroup),
            "scholarship" => Ok(Self::Scholarship),
            "internship" => Ok(Self::Internship),
            "career" => Ok(Self::Career),
            "student_life" => Ok(Self::StudentLife),
            "general" => Ok(Self::General),
            "other" => Ok(Self::Other),
            _ => Err(crate::enums::admin_auth_error::AdminAuthError::InvalidCommunityFilter),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunityVisibility {
    #[default]
    Public,
    Restricted,
    Private,
}

impl CommunityVisibility {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Restricted => "restricted",
            Self::Private => "private",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunityMembershipPolicy {
    #[default]
    Open,
    ApprovalRequired,
}

impl CommunityMembershipPolicy {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::ApprovalRequired => "approval_required",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommunityLifecycleStatus {
    Active,
    Suspended,
    Archived,
}

impl CommunityLifecycleStatus {
    pub const fn as_database_value(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for CommunityLifecycleStatus {
    type Error = crate::enums::admin_auth_error::AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "archived" => Ok(Self::Archived),
            _ => Err(crate::enums::admin_auth_error::AdminAuthError::InvalidCommunityFilter),
        }
    }
}
