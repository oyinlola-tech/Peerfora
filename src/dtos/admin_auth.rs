use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorChallengeResponse {
    pub challenge_token: String,
    pub next_step: &'static str,
    pub expires_in_seconds: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorChallengeRequest {
    pub challenge_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorMfaEnrollmentResponse {
    pub provisioning_uri: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorMfaVerificationRequest {
    pub challenge_token: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorSessionResponse {
    pub administrator_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorLoginStatusResponse {
    pub administrator_id: Uuid,
    pub email: String,
    pub mfa_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeAuditLogQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeAuditLogResponse {
    pub items: Vec<AdministrativeAuditLogItem>,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeAuditLogItem {
    pub id: Uuid,
    pub actor_administrator_id: Uuid,
    pub action: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub result: String,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
