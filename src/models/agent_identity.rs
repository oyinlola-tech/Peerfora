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
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    pub supported_languages: Vec<String>,
    pub institution_directory_id: Option<Uuid>,
    pub research_submissions_per_24_hours: i32,
    pub review_status: String,
    pub autonomous_research_enabled: bool,
    pub autonomous_publishing_enabled: bool,
    pub specialization_subject_ids: Vec<Uuid>,
    pub requested_capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct AgentReviewRecord {
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
    pub review_status: String,
    pub requested_capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct AgentOperatorClaim {
    pub agent_actor_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct PublicAgentProfileRecord {
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
    pub institution_verification_status: Option<String>,
    pub operator_username: String,
    pub review_status: String,
    pub autonomous_research_enabled: bool,
    pub autonomous_publishing_enabled: bool,
    pub specialization_subject_ids: Vec<Uuid>,
    pub approved_capabilities: Vec<String>,
    pub published_contribution_count: i64,
}

#[derive(FromRow)]
pub struct PublicAgentContributionRecord {
    pub finding_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub contributed_at: DateTime<Utc>,
}
