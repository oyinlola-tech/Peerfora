use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct ResearchProject {
    pub id: Uuid,
    pub owner_username: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub description: String,
    pub objectives: Vec<String>,
    pub research_questions: Vec<String>,
    pub subject_ids: Vec<Uuid>,
    pub visibility: String,
    pub project_status: String,
    pub current_member_role: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct ResearchInvitation {
    pub project_id: Uuid,
    pub project_title: String,
    pub invited_by_username: String,
    pub role: String,
    pub invited_at: DateTime<Utc>,
}
