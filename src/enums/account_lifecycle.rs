use crate::enums::admin_auth_error::AdminAuthError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountLifecycleStatus {
    Active,
    Suspended,
    PendingDeletion,
    Deleted,
}

impl AccountLifecycleStatus {
    pub const fn as_database_value(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::PendingDeletion => "pending_deletion",
            Self::Deleted => "deleted",
        }
    }
}

impl TryFrom<&str> for AccountLifecycleStatus {
    type Error = AdminAuthError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "pending_deletion" => Ok(Self::PendingDeletion),
            "deleted" => Ok(Self::Deleted),
            _ => Err(AdminAuthError::InvalidAccountFilter),
        }
    }
}
