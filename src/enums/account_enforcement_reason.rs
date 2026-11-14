use crate::enums::admin_auth_error::AdminAuthError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountEnforcementReason {
    PolicyViolation,
    UserSafety,
    AccountCompromise,
    LegalRequirement,
    Other,
}

impl AccountEnforcementReason {
    pub const fn as_database_value(self) -> &'static str {
        match self {
            Self::PolicyViolation => "policy_violation",
            Self::UserSafety => "user_safety",
            Self::AccountCompromise => "account_compromise",
            Self::LegalRequirement => "legal_requirement",
            Self::Other => "other",
        }
    }
}

impl TryFrom<&str> for AccountEnforcementReason {
    type Error = AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "policy_violation" => Ok(Self::PolicyViolation),
            "user_safety" => Ok(Self::UserSafety),
            "account_compromise" => Ok(Self::AccountCompromise),
            "legal_requirement" => Ok(Self::LegalRequirement),
            "other" => Ok(Self::Other),
            _ => Err(AdminAuthError::InvalidEnforcementReason),
        }
    }
}
