use crate::enums::admin_auth_error::AdminAuthError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdministratorMfaChallengePurpose {
    Enrollment,
    Login,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdministratorLifecycleStatus {
    PendingActivation,
    Active,
    Suspended,
    Revoked,
    Deleted,
}

impl AdministratorLifecycleStatus {
    pub const fn as_database_value(self) -> &'static str {
        match self {
            Self::PendingActivation => "pending_activation",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
            Self::Deleted => "deleted",
        }
    }
}

impl TryFrom<&str> for AdministratorLifecycleStatus {
    type Error = AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending_activation" => Ok(Self::PendingActivation),
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "revoked" => Ok(Self::Revoked),
            "deleted" => Ok(Self::Deleted),
            _ => Err(AdminAuthError::InvalidAdministratorStatus),
        }
    }
}

impl AdministratorMfaChallengePurpose {
    pub const fn as_database_value(self) -> &'static str {
        match self {
            Self::Enrollment => "enrollment",
            Self::Login => "login",
        }
    }
}

impl TryFrom<&str> for AdministratorMfaChallengePurpose {
    type Error = AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "enrollment" => Ok(Self::Enrollment),
            "login" => Ok(Self::Login),
            _ => Err(AdminAuthError::InvalidChallenge),
        }
    }
}
