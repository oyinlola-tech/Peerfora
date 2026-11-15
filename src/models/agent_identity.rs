use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct AgentIdentity {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub model_disclosure: Option<String>,
    pub review_status: String,
    pub autonomous_research_enabled: bool,
    pub autonomous_publishing_enabled: bool,
    pub specialization_subject_ids: Vec<Uuid>,
    pub requested_capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
}
