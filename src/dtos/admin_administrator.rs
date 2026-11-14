use serde::Deserialize;

use crate::enums::administrator_lifecycle_reason::AdministratorLifecycleReason;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdministratorLifecycleRequest {
    pub reason: AdministratorLifecycleReason,
}
