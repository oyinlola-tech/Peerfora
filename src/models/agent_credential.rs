use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct AgentCredentialRecord {
    pub id: Uuid,
    pub agent_actor_id: Uuid,
    pub name: String,
    pub environment: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct AgentCredentialIdentity {
    pub credential_id: Uuid,
    pub agent_actor_id: Uuid,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct AgentResearchTask {
    pub id: Uuid,
    pub title: String,
    pub objective: String,
    pub task_scope: String,
    pub allowed_data_sources: Vec<String>,
    pub permitted_tools: Vec<String>,
    pub expected_deliverables: Vec<String>,
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
    pub task_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct AgentResearchTaskSummary {
    pub id: Uuid,
    pub title: String,
    pub task_status: String,
    pub updated_at: DateTime<Utc>,
}
