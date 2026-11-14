use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdministratorLifecycleReason {
    SecurityConcern,
    PolicyViolation,
    RoleChange,
    Departure,
    Other,
}

impl AdministratorLifecycleReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SecurityConcern => "security_concern",
            Self::PolicyViolation => "policy_violation",
            Self::RoleChange => "role_change",
            Self::Departure => "departure",
            Self::Other => "other",
        }
    }
}
