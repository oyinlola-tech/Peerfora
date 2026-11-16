use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    constants::research::{DEFAULT_RESEARCH_FINDING_LIST_LIMIT, MAX_RESEARCH_FINDING_LIST_LIMIT},
    enums::research::{ResearchFindingReviewDecision, ResearchSourceType},
    models::research_finding::{
        ResearchFinding, ResearchFindingCitation, ResearchFindingRevision, ResearchSource,
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResearchSourceRequest {
    pub source_type: ResearchSourceType,
    pub title: String,
    pub doi: Option<String>,
    pub source_url: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub publication_year: Option<i16>,
    pub provenance: String,
    pub license: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResearchFindingRequest {
    pub title: String,
    pub statement: String,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub citations: Vec<ResearchFindingCitationRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResearchFindingRequest {
    pub title: String,
    pub statement: String,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub citations: Vec<ResearchFindingCitationRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchFindingCitationRequest {
    pub source_id: Uuid,
    pub locator: Option<String>,
    pub context: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResearchFindingRequest {
    pub decision: ResearchFindingReviewDecision,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ListResearchFindingsQuery {
    pub limit: Option<i64>,
}

impl ListResearchFindingsQuery {
    pub fn normalized_limit(&self) -> i64 {
        self.limit
            .unwrap_or(DEFAULT_RESEARCH_FINDING_LIST_LIMIT)
            .clamp(1, MAX_RESEARCH_FINDING_LIST_LIMIT)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchSourceResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub source_type: String,
    pub title: String,
    pub doi: Option<String>,
    pub source_url: Option<String>,
    pub authors: Vec<String>,
    pub publication_year: Option<i16>,
    pub provenance: String,
    pub license: Option<String>,
    pub created_by_username: Option<String>,
    pub created_by_agent_handle: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ResearchSource> for ResearchSourceResponse {
    fn from(source: ResearchSource) -> Self {
        Self {
            id: source.id,
            project_id: source.project_id,
            source_type: source.source_type,
            title: source.title,
            doi: source.doi,
            source_url: source.source_url,
            authors: source.authors,
            publication_year: source.publication_year,
            provenance: source.provenance,
            license: source.license,
            created_by_username: source.created_by_username,
            created_by_agent_handle: source.created_by_agent_handle,
            created_at: source.created_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchFindingResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by_username: String,
    pub authored_by_username: Option<String>,
    pub authored_by_agent_handle: Option<String>,
    pub status: String,
    pub revision_number: i16,
    pub published_revision_number: Option<i16>,
    pub title: String,
    pub statement: String,
    pub details: String,
    pub citations: Vec<ResearchFindingCitation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ResearchFinding> for ResearchFindingResponse {
    fn from(finding: ResearchFinding) -> Self {
        Self {
            id: finding.id,
            project_id: finding.project_id,
            created_by_username: finding.created_by_username,
            authored_by_username: finding.authored_by_username,
            authored_by_agent_handle: finding.authored_by_agent_handle,
            status: finding.finding_status,
            revision_number: finding.revision_number,
            published_revision_number: finding.published_revision_number,
            title: finding.title,
            statement: finding.statement,
            details: finding.details,
            citations: finding.citations.0,
            created_at: finding.created_at,
            updated_at: finding.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchFindingRevisionResponse {
    pub finding_id: Uuid,
    pub project_id: Uuid,
    pub revision_number: i16,
    pub title: String,
    pub statement: String,
    pub details: String,
    pub authored_by_username: Option<String>,
    pub authored_by_agent_handle: Option<String>,
    pub review_status: String,
    pub reviewed_by_username: Option<String>,
    pub reviewer_notes: Option<String>,
    pub citations: Vec<ResearchFindingCitation>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

impl From<ResearchFindingRevision> for ResearchFindingRevisionResponse {
    fn from(revision: ResearchFindingRevision) -> Self {
        Self {
            finding_id: revision.finding_id,
            project_id: revision.project_id,
            revision_number: revision.revision_number,
            title: revision.title,
            statement: revision.statement,
            details: revision.details,
            authored_by_username: revision.authored_by_username,
            authored_by_agent_handle: revision.authored_by_agent_handle,
            review_status: revision.review_status,
            reviewed_by_username: revision.reviewed_by_username,
            reviewer_notes: revision.reviewer_notes,
            citations: revision.citations.0,
            created_at: revision.created_at,
            reviewed_at: revision.reviewed_at,
        }
    }
}
