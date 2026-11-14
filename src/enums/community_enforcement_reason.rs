use crate::enums::admin_auth_error::AdminAuthError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommunityEnforcementReason {
    PolicyViolation,
    UserSafety,
    LegalRequirement,
    Other,
}

impl CommunityEnforcementReason {
    pub const fn as_database_value(self) -> &'static str {
        match self {
            Self::PolicyViolation => "policy_violation",
            Self::UserSafety => "user_safety",
            Self::LegalRequirement => "legal_requirement",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for CommunityEnforcementReason {
    type Error = AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "policy_violation" => Ok(Self::PolicyViolation),
            "user_safety" => Ok(Self::UserSafety),
            "legal_requirement" => Ok(Self::LegalRequirement),
            "other" => Ok(Self::Other),
            _ => Err(AdminAuthError::InvalidCommunityEnforcementReason),
        }
    }
}
