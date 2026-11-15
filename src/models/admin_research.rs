use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::enums::research::ResearchProjectStatus;

#[derive(FromRow)]
pub struct AdministrativeResearchProjectRecord {
    pub id: Uuid,
    pub owner_username: String,
    pub title: String,
    pub visibility: String,
    pub project_status: String,
    pub active_member_count: i64,
    pub finding_count: i64,
    pub task_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct AdministrativeResearchProjectFilters<'a> {
    pub search_prefix: Option<&'a str>,
    pub project_status: Option<ResearchProjectStatus>,
}
