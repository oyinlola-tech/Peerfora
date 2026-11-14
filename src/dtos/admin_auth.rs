use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdministratorPasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdministratorPasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdministratorPasswordResetConfirmationRequest {
    pub token: String,
    pub new_password: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdministratorRecoveryCodeRegenerationRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorRecoveryCodesResponse {
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorSessionResponse {
    pub administrator_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_codes: Option<Vec<String>>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorSessionsResponse {
    pub items: Vec<AdministratorSessionItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorSessionItem {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorOverviewResponse {
    pub registered_accounts: i64,
    pub active_accounts: i64,
    pub pending_email_verification: i64,
    pub suspended_accounts: i64,
    pub total_institutions: i64,
    pub total_communities: i64,
    pub active_communities: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeAccountQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub country_code: Option<String>,
    pub academic_level_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeAccountsResponse {
    pub items: Vec<AdministrativeAccountItem>,
    pub total_count: i64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeAccountItem {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub status: String,
    pub email_verified: bool,
    pub country_code: String,
    pub academic_level_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountEnforcementRequest {
    pub reason_code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSessionsRevocationResponse {
    pub revoked_session_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeCommunityQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeCommunitiesResponse {
    pub items: Vec<AdministrativeCommunityItem>,
    pub total_count: i64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeCommunityItem {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub category: String,
    pub visibility: String,
    pub lifecycle_status: String,
    pub owner_account_id: Uuid,
    pub member_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityEnforcementRequest {
    pub reason_code: String,
}
