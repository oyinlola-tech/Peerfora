use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::agent::{
    AgentCapability, AgentLifecycleAction, AgentOperatorVerificationStatus, AgentReviewDecision,
    AgentReviewStatus, InstitutionDirectoryVerificationStatus,
};
use crate::{
    errors::app_error::AppError,
    models::agent_identity::{
        AgentIdentity, PublicAgentContributionRecord, PublicAgentProfileRecord,
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAgentRequest {
    pub handle: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub model_disclosure: Option<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    #[serde(default)]
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    #[serde(default)]
    pub specialization_subject_ids: Vec<Uuid>,
    #[serde(default)]
    pub requested_capabilities: Vec<AgentCapability>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitiateAgentRegistrationRequest {
    pub operator_email: String,
    pub handle: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub model_disclosure: Option<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    #[serde(default)]
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    #[serde(default)]
    pub specialization_subject_ids: Vec<Uuid>,
    #[serde(default)]
    pub requested_capabilities: Vec<AgentCapability>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteAgentOperatorClaimRequest {
    pub token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMyAgentsQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentReviewQuery {
    pub status: Option<AgentReviewStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewAgentRequest {
    pub decision: AgentReviewDecision,
    #[serde(default)]
    pub approved_capabilities: Vec<AgentCapability>,
    pub reason: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdministrateAgentRequest {
    pub action: AgentLifecycleAction,
    pub reason: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateAgentResearchQuotaRequest {
    pub submissions_per_24_hours: i32,
    pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentIdentityResponse {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    pub research_submissions_per_24_hours: i32,
    pub review_status: AgentReviewStatus,
    pub autonomous_research_enabled: bool,
    pub autonomous_publishing_enabled: bool,
    pub specialization_subject_ids: Vec<Uuid>,
    pub requested_capabilities: Vec<AgentCapability>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewItem {
    pub id: Uuid,
    pub operator_account_id: Option<Uuid>,
    pub operator_username: Option<String>,
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    pub research_submissions_per_24_hours: i32,
    pub review_status: AgentReviewStatus,
    pub requested_capabilities: Vec<AgentCapability>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAgentProfileResponse {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    pub institution_directory_name: Option<String>,
    pub institution_directory_verification_status: Option<InstitutionDirectoryVerificationStatus>,
    pub operator_username: String,
    pub operator_verification_status: AgentOperatorVerificationStatus,
    pub operational_status: AgentReviewStatus,
    pub autonomous_research_enabled: bool,
    pub autonomous_publishing_enabled: bool,
    pub specialization_subject_ids: Vec<Uuid>,
    pub approved_capabilities: Vec<AgentCapability>,
    pub published_contribution_count: i64,
    pub recent_contributions: Vec<PublicAgentContributionResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAgentContributionResponse {
    pub finding_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub contributed_at: DateTime<Utc>,
}

impl From<PublicAgentContributionRecord> for PublicAgentContributionResponse {
    fn from(record: PublicAgentContributionRecord) -> Self {
        Self {
            finding_id: record.finding_id,
            project_id: record.project_id,
            title: record.title,
            contributed_at: record.contributed_at,
        }
    }
}

impl TryFrom<(PublicAgentProfileRecord, Vec<PublicAgentContributionRecord>)>
    for PublicAgentProfileResponse
{
    type Error = AppError;

    fn try_from(
        (profile, contributions): (PublicAgentProfileRecord, Vec<PublicAgentContributionRecord>),
    ) -> Result<Self, Self::Error> {
        let operational_status = AgentReviewStatus::from_database_value(&profile.review_status)
            .ok_or(AppError::Internal)?;
        let approved_capabilities = profile
            .approved_capabilities
            .iter()
            .map(|value| AgentCapability::from_database_value(value).ok_or(AppError::Internal))
            .collect::<Result<Vec<_>, _>>()?;
        let institution_directory_verification_status =
            match profile.institution_verification_status.as_deref() {
                Some(value) => Some(
                    InstitutionDirectoryVerificationStatus::from_database_value(value)
                        .ok_or(AppError::Internal)?,
                ),
                None => None,
            };
        Ok(Self {
            id: profile.id,
            handle: profile.handle,
            display_name: profile.display_name,
            description: profile.description,
            model_disclosure: profile.model_disclosure,
            model_name: profile.model_name,
            model_version: profile.model_version,
            supported_languages: profile.supported_languages,
            institution_directory_id: profile.institution_directory_id,
            institution_directory_name: profile.institution_directory_name,
            institution_directory_verification_status,
            operator_username: profile.operator_username,
            operator_verification_status:
                AgentOperatorVerificationStatus::EmailClaimedByActiveVerifiedAccount,
            operational_status,
            autonomous_research_enabled: profile.autonomous_research_enabled,
            autonomous_publishing_enabled: profile.autonomous_publishing_enabled,
            specialization_subject_ids: profile.specialization_subject_ids,
            approved_capabilities,
            published_contribution_count: profile.published_contribution_count,
            recent_contributions: contributions.into_iter().map(Into::into).collect(),
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewListResponse {
    pub items: Vec<AgentReviewItem>,
    pub total_count: i64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClaimInitiatedResponse {
    pub agent_id: Uuid,
    pub review_status: AgentReviewStatus,
    pub claim_expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClaimCompletedResponse {
    pub agent_id: Uuid,
    pub review_status: AgentReviewStatus,
}

impl TryFrom<crate::models::agent_identity::AgentReviewRecord> for AgentReviewItem {
    type Error = AppError;

    fn try_from(
        record: crate::models::agent_identity::AgentReviewRecord,
    ) -> Result<Self, Self::Error> {
        let review_status = AgentReviewStatus::from_database_value(&record.review_status)
            .ok_or(AppError::Internal)?;
        let requested_capabilities = record
            .requested_capabilities
            .iter()
            .map(|value| AgentCapability::from_database_value(value).ok_or(AppError::Internal))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id: record.id,
            operator_account_id: record.operator_account_id,
            operator_username: record.operator_username,
            handle: record.handle,
            display_name: record.display_name,
            description: record.description,
            model_disclosure: record.model_disclosure,
            model_name: record.model_name,
            model_version: record.model_version,
            supported_languages: record.supported_languages,
            institution_directory_id: record.institution_directory_id,
            research_submissions_per_24_hours: record.research_submissions_per_24_hours,
            review_status,
            requested_capabilities,
            created_at: record.created_at,
        })
    }
}

impl TryFrom<AgentIdentity> for AgentIdentityResponse {
    type Error = AppError;

    fn try_from(identity: AgentIdentity) -> Result<Self, Self::Error> {
        let review_status = AgentReviewStatus::from_database_value(&identity.review_status)
            .ok_or(AppError::Internal)?;
        let requested_capabilities = identity
            .requested_capabilities
            .iter()
            .map(|value| AgentCapability::from_database_value(value).ok_or(AppError::Internal))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            id: identity.id,
            handle: identity.handle,
            display_name: identity.display_name,
            description: identity.description,
            model_disclosure: identity.model_disclosure,
            model_name: identity.model_name,
            model_version: identity.model_version,
            supported_languages: identity.supported_languages,
            institution_directory_id: identity.institution_directory_id,
            research_submissions_per_24_hours: identity.research_submissions_per_24_hours,
            review_status,
            autonomous_research_enabled: identity.autonomous_research_enabled,
            autonomous_publishing_enabled: identity.autonomous_publishing_enabled,
            specialization_subject_ids: identity.specialization_subject_ids,
            requested_capabilities,
            created_at: identity.created_at,
        })
    }
}
