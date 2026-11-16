use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct KnowledgeEntrySummary {
    pub id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct KnowledgeCitationRecord {
    pub title: String,
    pub doi: Option<String>,
    pub external_url: Option<String>,
    pub authors: Value,
    pub publication_year: Option<i16>,
    pub license: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct KnowledgeFindingReferenceRecord {
    pub finding_id: Uuid,
    pub relation: String,
    pub title: String,
}

#[derive(sqlx::FromRow)]
pub struct KnowledgeFindingProposalRecord {
    pub finding_id: Uuid,
    pub relation: String,
    pub public_title: Option<String>,
    pub currently_public: bool,
}

#[derive(sqlx::FromRow)]
pub struct KnowledgeProposalDetailRecord {
    pub id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct KnowledgeEntryRecord {
    pub id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

pub struct KnowledgeCitationInputRecord<'a> {
    pub title: &'a str,
    pub doi: Option<&'a str>,
    pub external_url: Option<&'a str>,
    pub authors: &'a [String],
    pub publication_year: Option<i16>,
    pub license: Option<&'a str>,
}
