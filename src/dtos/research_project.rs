use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::{
    enums::research::ResearchProjectMemberRole, models::research_project::ResearchProject,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResearchProjectRequest {
    pub title: String,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub objectives: Vec<String>,
    #[serde(default)]
    pub research_questions: Vec<String>,
    #[serde(default)]
    pub subject_ids: Vec<Uuid>,
    #[serde(default)]
    pub visibility: crate::enums::research::ResearchProjectVisibility,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResearchProjectRequest {
    pub title: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_nullable_field",
        rename = "abstract"
    )]
    pub abstract_text: Option<Option<String>>,
    pub description: Option<String>,
    pub objectives: Option<Vec<String>>,
    pub research_questions: Option<Vec<String>>,
    pub subject_ids: Option<Vec<Uuid>>,
}

fn deserialize_nullable_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteResearchProjectMemberRequest {
    pub username: String,
    pub role: ResearchProjectMemberRole,
}

#[derive(Deserialize)]
pub struct ListResearchProjectsQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchProjectResponse {
    pub id: Uuid,
    pub owner_username: String,
    pub title: String,
    #[serde(rename = "abstract")]
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

impl From<ResearchProject> for ResearchProjectResponse {
    fn from(project: ResearchProject) -> Self {
        Self {
            id: project.id,
            owner_username: project.owner_username,
            title: project.title,
            abstract_text: project.abstract_text,
            description: project.description,
            objectives: project.objectives,
            research_questions: project.research_questions,
            subject_ids: project.subject_ids,
            visibility: project.visibility,
            project_status: project.project_status,
            current_member_role: project.current_member_role,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchInvitationResponse {
    pub project_id: Uuid,
    pub project_title: String,
    pub invited_by_username: String,
    pub role: String,
    pub invited_at: DateTime<Utc>,
}

impl From<crate::models::research_project::ResearchInvitation> for ResearchInvitationResponse {
    fn from(invitation: crate::models::research_project::ResearchInvitation) -> Self {
        Self {
            project_id: invitation.project_id,
            project_title: invitation.project_title,
            invited_by_username: invitation.invited_by_username,
            role: invitation.role,
            invited_at: invitation.invited_at,
        }
    }
}
