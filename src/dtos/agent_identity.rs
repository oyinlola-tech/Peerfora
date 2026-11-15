use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::agent::{AgentCapability, AgentReviewStatus};
use crate::{errors::app_error::AppError, models::agent_identity::AgentIdentity};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAgentRequest {
    pub handle: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub model_disclosure: Option<String>,
    #[serde(default)]
    pub specialization_subject_ids: Vec<Uuid>,
    #[serde(default)]
    pub requested_capabilities: Vec<AgentCapability>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMyAgentsQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentIdentityResponse {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub review_status: AgentReviewStatus,
    pub autonomous_research_enabled: bool,
    pub autonomous_publishing_enabled: bool,
    pub specialization_subject_ids: Vec<Uuid>,
    pub requested_capabilities: Vec<AgentCapability>,
    pub created_at: DateTime<Utc>,
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
            review_status,
            autonomous_research_enabled: identity.autonomous_research_enabled,
            autonomous_publishing_enabled: identity.autonomous_publishing_enabled,
            specialization_subject_ids: identity.specialization_subject_ids,
            requested_capabilities,
            created_at: identity.created_at,
        })
    }
}
