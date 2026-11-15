use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::research::{
    DEFAULT_RESEARCH_TASK_RESULT_BYTES, DEFAULT_RESEARCH_TASK_RUNTIME_SECONDS,
};
use crate::models::research_task::ResearchTask;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResearchTaskRequest {
    pub title: String,
    pub objective: String,
    pub scope: String,
    #[serde(default)]
    pub allowed_data_sources: Vec<String>,
    #[serde(default)]
    pub permitted_tools: Vec<String>,
    pub expected_deliverables: Vec<String>,
    #[serde(default = "default_max_runtime_seconds")]
    pub max_runtime_seconds: i32,
    #[serde(default = "default_max_result_bytes")]
    pub max_result_bytes: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResearchTaskRequest {
    pub title: String,
    pub objective: String,
    pub scope: String,
    #[serde(default)]
    pub allowed_data_sources: Vec<String>,
    #[serde(default)]
    pub permitted_tools: Vec<String>,
    pub expected_deliverables: Vec<String>,
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignResearchTaskRequest {
    pub assignee_username: String,
}

#[derive(Deserialize)]
pub struct SubmitResearchTaskResultRequest {
    pub result: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResearchTaskResultRequest {
    pub decision: crate::enums::research::ResearchTaskReviewDecision,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ListResearchTasksQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchTaskResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by_username: String,
    pub title: String,
    pub objective: String,
    pub scope: String,
    pub allowed_data_sources: Vec<String>,
    pub permitted_tools: Vec<String>,
    pub expected_deliverables: Vec<String>,
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
    pub status: String,
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

impl From<ResearchTask> for ResearchTaskResponse {
    fn from(task: ResearchTask) -> Self {
        Self {
            id: task.id,
            project_id: task.project_id,
            created_by_username: task.created_by_username,
            title: task.title,
            objective: task.objective,
            scope: task.task_scope,
            allowed_data_sources: task.allowed_data_sources,
            permitted_tools: task.permitted_tools,
            expected_deliverables: task.expected_deliverables,
            max_runtime_seconds: task.max_runtime_seconds,
            max_result_bytes: task.max_result_bytes,
            status: task.task_status,
            assigned_username: task.assigned_username,
            assignment_accepted_at: task.assignment_accepted_at,
            latest_submission_number: task.latest_submission_number,
            latest_result: task.latest_result,
            latest_submission_by_username: task.latest_submission_by_username,
            latest_review_status: task.latest_review_status,
            latest_reviewed_by_username: task.latest_reviewed_by_username,
            latest_review_notes: task.latest_review_notes,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}

fn default_max_runtime_seconds() -> i32 {
    DEFAULT_RESEARCH_TASK_RUNTIME_SECONDS
}

fn default_max_result_bytes() -> i32 {
    DEFAULT_RESEARCH_TASK_RESULT_BYTES
}
