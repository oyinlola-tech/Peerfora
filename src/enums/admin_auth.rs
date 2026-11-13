use crate::enums::admin_auth_error::AdminAuthError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdministratorMfaChallengePurpose {
    Enrollment,
    Login,
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
