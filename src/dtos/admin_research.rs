use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    constants::admin_auth::{ADMIN_RESEARCH_PROJECTS_DEFAULT_PAGE_SIZE, ADMIN_RESEARCH_PROJECTS_MAX_PAGE_SIZE},
    models::admin_research::AdministrativeResearchProjectRecord,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeResearchProjectQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl AdministrativeResearchProjectQuery {
    pub fn normalized_limit(&self) -> u32 {
        self.limit
            .unwrap_or(ADMIN_RESEARCH_PROJECTS_DEFAULT_PAGE_SIZE)
            .clamp(1, ADMIN_RESEARCH_PROJECTS_MAX_PAGE_SIZE)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchProjectEnforcementRequest {
    pub reason_code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeResearchProjectsResponse {
    pub items: Vec<AdministrativeResearchProjectItem>,
    pub total_count: i64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeResearchProjectItem {
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

impl From<AdministrativeResearchProjectRecord> for AdministrativeResearchProjectItem {
    fn from(project: AdministrativeResearchProjectRecord) -> Self {
        Self {
            id: project.id,
            owner_username: project.owner_username,
            title: project.title,
            visibility: project.visibility,
            project_status: project.project_status,
            active_member_count: project.active_member_count,
            finding_count: project.finding_count,
            task_count: project.task_count,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}
