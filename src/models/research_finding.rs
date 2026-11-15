use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(FromRow)]
pub struct ResearchSource {
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
    pub created_by_username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchFindingCitation {
    pub source_id: Uuid,
    pub source_type: String,
    pub title: String,
    pub doi: Option<String>,
    pub source_url: Option<String>,
    pub authors: Vec<String>,
    pub publication_year: Option<i16>,
    pub provenance: String,
    pub license: Option<String>,
    pub locator: Option<String>,
    pub context: Option<String>,
}

#[derive(FromRow)]
pub struct ResearchFinding {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by_username: String,
    pub authored_by_username: String,
    pub finding_status: String,
    pub revision_number: i16,
    pub published_revision_number: Option<i16>,
    pub title: String,
    pub statement: String,
    pub details: String,
    pub citations: Json<Vec<ResearchFindingCitation>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
