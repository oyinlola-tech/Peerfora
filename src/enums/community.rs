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
