use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    enums::agent::{AgentCapability, AgentCredentialEnvironment},
    errors::app_error::AppError,
    models::agent_credential::AgentCredentialRecord,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IssueAgentCredentialRequest {
    pub name: String,
    pub capabilities: Vec<AgentCapability>,
    pub expires_in_days: Option<u16>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RotateAgentCredentialRequest {
    pub expires_in_days: Option<u16>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCredentialItem {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub name: String,
    pub environment: AgentCredentialEnvironment,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub capabilities: Vec<AgentCapability>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCredentialListResponse {
    pub items: Vec<AgentCredentialItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuedAgentCredentialResponse {
    pub credential: AgentCredentialItem,
    pub secret: String,
}

impl TryFrom<AgentCredentialRecord> for AgentCredentialItem {
    type Error = AppError;

    fn try_from(record: AgentCredentialRecord) -> Result<Self, Self::Error> {
        let environment = AgentCredentialEnvironment::from_database_value(&record.environment)
            .ok_or(AppError::Internal)?;
        let capabilities = record
            .capabilities
            .iter()
            .map(|capability| {
                AgentCapability::from_database_value(capability).ok_or(AppError::Internal)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id: record.id,
            agent_id: record.agent_actor_id,
            name: record.name,
            environment,
            created_at: record.created_at,
            expires_at: record.expires_at,
            revoked_at: record.revoked_at,
            last_used_at: record.last_used_at,
            capabilities,
        })
    }
}
use crate::models::agent_credential::{AgentResearchTask, AgentResearchTaskSummary};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResearchTaskResponse {
    pub id: Uuid,
    pub title: String,
    pub objective: String,
    pub scope: String,
    pub allowed_data_sources: Vec<String>,
    pub permitted_tools: Vec<String>,
    pub expected_deliverables: Vec<String>,
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AgentResearchTask> for AgentResearchTaskResponse {
    fn from(task: AgentResearchTask) -> Self {
        Self {
            id: task.id,
            title: task.title,
            objective: task.objective,
            scope: task.task_scope,
            allowed_data_sources: task.allowed_data_sources,
            permitted_tools: task.permitted_tools,
            expected_deliverables: task.expected_deliverables,
            max_runtime_seconds: task.max_runtime_seconds,
            max_result_bytes: task.max_result_bytes,
            status: task.task_status,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResearchTaskSummaryResponse {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

impl From<AgentResearchTaskSummary> for AgentResearchTaskSummaryResponse {
    fn from(task: AgentResearchTaskSummary) -> Self {
        Self {
            id: task.id,
            title: task.title,
            status: task.task_status,
            updated_at: task.updated_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResearchTaskSubmissionRequest {
    pub result: String,
    pub sources: Vec<AgentResearchTaskSourceRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResearchTaskSourceRequest {
    pub title: String,
    pub url: String,
    pub doi: Option<String>,
}
