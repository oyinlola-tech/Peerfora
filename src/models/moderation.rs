use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub enum ModerationReportTarget {
    Account(Uuid),
    Community(Uuid),
    Post(Uuid),
    Comment(Uuid),
    ResearchProject(Uuid),
    ResearchFinding(Uuid),
    Conversation(Uuid),
    Message(Uuid),
    Media(Uuid),
}

#[derive(sqlx::FromRow)]
pub struct ModerationReportPage {
    pub id: Uuid,
    pub reporter_account_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub category: String,
    pub description: String,
    pub status: String,
    pub assigned_administrator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub total_count: i64,
}
