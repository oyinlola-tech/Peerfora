use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAdministratorInvitationRequest {
    pub email: String,
    pub role_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptAdministratorInvitationRequest {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptAdministratorInvitationResponse {
    pub administrator_id: Uuid,
    pub assigned_role: String,
    pub next_step: &'static str,
}
