use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct ResearchTask {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by_username: String,
    pub title: String,
    pub objective: String,
    pub task_scope: String,
    pub allowed_data_sources: Vec<String>,
    pub permitted_tools: Vec<String>,
    pub expected_deliverables: Vec<String>,
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
    pub task_status: String,
    pub assigned_username: Option<String>,
    pub assignment_accepted_at: Option<DateTime<Utc>>,
    pub latest_submission_number: Option<i16>,
    pub latest_result: Option<String>,
    pub latest_submission_by_username: Option<String>,
    pub latest_review_status: Option<String>,
    pub latest_reviewed_by_username: Option<String>,
    pub latest_review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
