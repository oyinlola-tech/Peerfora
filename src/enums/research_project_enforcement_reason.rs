use crate::enums::admin_auth_error::AdminAuthError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchProjectEnforcementReason {
    IntegrityConcern,
    PolicyViolation,
    PrivacyConcern,
    LegalRequirement,
    Other,
}

impl ResearchProjectEnforcementReason {
    pub const fn as_database_value(self) -> &'static str {
        match self {
            Self::IntegrityConcern => "integrity_concern",
            Self::PolicyViolation => "policy_violation",
            Self::PrivacyConcern => "privacy_concern",
            Self::LegalRequirement => "legal_requirement",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for ResearchProjectEnforcementReason {
    type Error = AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "integrity_concern" => Ok(Self::IntegrityConcern),
            "policy_violation" => Ok(Self::PolicyViolation),
            "privacy_concern" => Ok(Self::PrivacyConcern),
            "legal_requirement" => Ok(Self::LegalRequirement),
            "other" => Ok(Self::Other),
            _ => Err(AdminAuthError::InvalidResearchEnforcementReason),
        }
    }
}
