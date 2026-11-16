use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::moderation::{
    ModerationReportCategory, ModerationReportDecision, ModerationReportStatus,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateModerationReportRequest {
    pub category: ModerationReportCategory,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveModerationReportRequest {
    pub decision: ModerationReportDecision,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssignModerationReportRequest {
    pub administrator_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeModerationReportQuery {
    pub status: Option<ModerationReportStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationReportSubmissionResponse {
    pub id: Uuid,
    pub status: &'static str,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeModerationReportsResponse {
    pub items: Vec<AdministrativeModerationReportItem>,
    pub total_count: i64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeModerationReportItem {
    pub id: Uuid,
    pub reporter_account_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub category: String,
    pub description: String,
    pub status: String,
    pub assigned_administrator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<crate::models::moderation::ModerationReportPage> for AdministrativeModerationReportItem {
    fn from(report: crate::models::moderation::ModerationReportPage) -> Self {
        Self {
            id: report.id,
            reporter_account_id: report.reporter_account_id,
            target_type: report.target_type,
            target_id: report.target_id,
            category: report.category,
            description: report.description,
            status: report.status,
            assigned_administrator_id: report.assigned_administrator_id,
            created_at: report.created_at,
        }
    }
}
