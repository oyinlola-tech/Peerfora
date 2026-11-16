use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    constants::knowledge::KNOWLEDGE_CITATION_PROVENANCE,
    enums::knowledge::{KnowledgeEvidenceRelation, KnowledgeReviewDecision},
    errors::app_error::AppError,
    models::knowledge::{
        KnowledgeCitationRecord, KnowledgeEntryRecord, KnowledgeEntrySummary,
        KnowledgeFindingProposalRecord, KnowledgeFindingReferenceRecord,
        KnowledgeProposalDetailRecord,
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateKnowledgeProposalRequest {
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub citations: Vec<KnowledgeCitationInput>,
    #[serde(default)]
    pub research_findings: Vec<KnowledgeFindingInput>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeCitationInput {
    pub title: String,
    pub doi: Option<String>,
    pub external_url: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub publication_year: Option<i16>,
    pub license: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeFindingInput {
    pub finding_id: Uuid,
    pub relation: KnowledgeEvidenceRelation,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeProposalReviewRequest {
    pub decision: KnowledgeReviewDecision,
    pub reason: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeEntryPublishRequest {
    pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeProposalCreatedResponse {
    pub id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub status: &'static str,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListKnowledgeEntriesQuery {
    pub query: Option<String>,
    pub academic_subject_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListKnowledgeProposalsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEntrySummaryResponse {
    pub id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCitationResponse {
    pub title: String,
    pub doi: Option<String>,
    pub external_url: Option<String>,
    pub authors: Vec<String>,
    pub publication_year: Option<i16>,
    pub license: Option<String>,
    pub provenance: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeFindingReferenceResponse {
    pub finding_id: Uuid,
    pub relation: KnowledgeEvidenceRelation,
    pub title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEntryResponse {
    pub id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub status: &'static str,
    pub created_at: DateTime<Utc>,
    pub citations: Vec<KnowledgeCitationResponse>,
    pub research_findings: Vec<KnowledgeFindingReferenceResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeProposalReviewResponse {
    pub id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: String,
    pub summary: String,
    pub status: &'static str,
    pub created_at: DateTime<Utc>,
    pub citations: Vec<KnowledgeCitationResponse>,
    pub research_findings: Vec<KnowledgeFindingProposalResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeFindingProposalResponse {
    pub finding_id: Uuid,
    pub relation: KnowledgeEvidenceRelation,
    pub public_title: Option<String>,
    pub currently_public: bool,
}

impl From<KnowledgeEntrySummary> for KnowledgeEntrySummaryResponse {
    fn from(record: KnowledgeEntrySummary) -> Self {
        Self {
            id: record.id,
            academic_subject_id: record.academic_subject_id,
            title: record.title,
            summary: record.summary,
            created_at: record.created_at,
        }
    }
}

impl From<KnowledgeEntrySummary> for KnowledgeProposalCreatedResponse {
    fn from(record: KnowledgeEntrySummary) -> Self {
        Self {
            id: record.id,
            academic_subject_id: record.academic_subject_id,
            title: record.title,
            summary: record.summary,
            status: "submitted",
            created_at: record.created_at,
        }
    }
}

impl
    TryFrom<(
        KnowledgeEntryRecord,
        Vec<KnowledgeCitationRecord>,
        Vec<KnowledgeFindingReferenceRecord>,
    )> for KnowledgeEntryResponse
{
    type Error = AppError;

    fn try_from(
        (record, citations, research_findings): (
            KnowledgeEntryRecord,
            Vec<KnowledgeCitationRecord>,
            Vec<KnowledgeFindingReferenceRecord>,
        ),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: record.id,
            academic_subject_id: record.academic_subject_id,
            title: record.title,
            summary: record.summary,
            status: "published",
            created_at: record.created_at,
            citations: citations
                .into_iter()
                .map(KnowledgeCitationResponse::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            research_findings: research_findings
                .into_iter()
                .map(KnowledgeFindingReferenceResponse::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl
    TryFrom<(
        KnowledgeProposalDetailRecord,
        Vec<KnowledgeCitationRecord>,
        Vec<KnowledgeFindingProposalRecord>,
    )> for KnowledgeProposalReviewResponse
{
    type Error = AppError;

    fn try_from(
        (record, citations, research_findings): (
            KnowledgeProposalDetailRecord,
            Vec<KnowledgeCitationRecord>,
            Vec<KnowledgeFindingProposalRecord>,
        ),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: record.id,
            academic_subject_id: record.academic_subject_id,
            title: record.title,
            summary: record.summary,
            status: "submitted",
            created_at: record.created_at,
            citations: citations
                .into_iter()
                .map(KnowledgeCitationResponse::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            research_findings: research_findings
                .into_iter()
                .map(KnowledgeFindingProposalResponse::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<KnowledgeCitationRecord> for KnowledgeCitationResponse {
    type Error = AppError;

    fn try_from(record: KnowledgeCitationRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            title: record.title,
            doi: record.doi,
            external_url: record.external_url,
            authors: serde_json::from_value(record.authors).map_err(|_| AppError::Internal)?,
            publication_year: record.publication_year,
            license: record.license,
            provenance: KNOWLEDGE_CITATION_PROVENANCE,
        })
    }
}

impl TryFrom<KnowledgeFindingReferenceRecord> for KnowledgeFindingReferenceResponse {
    type Error = AppError;

    fn try_from(record: KnowledgeFindingReferenceRecord) -> Result<Self, Self::Error> {
        let relation = match record.relation.as_str() {
            "supports" => KnowledgeEvidenceRelation::Supports,
            "contradicts" => KnowledgeEvidenceRelation::Contradicts,
            "contextualizes" => KnowledgeEvidenceRelation::Contextualizes,
            _ => return Err(AppError::Internal),
        };
        Ok(Self {
            finding_id: record.finding_id,
            relation,
            title: record.title,
        })
    }
}

impl TryFrom<KnowledgeFindingProposalRecord> for KnowledgeFindingProposalResponse {
    type Error = AppError;

    fn try_from(record: KnowledgeFindingProposalRecord) -> Result<Self, Self::Error> {
        let relation = match record.relation.as_str() {
            "supports" => KnowledgeEvidenceRelation::Supports,
            "contradicts" => KnowledgeEvidenceRelation::Contradicts,
            "contextualizes" => KnowledgeEvidenceRelation::Contextualizes,
            _ => return Err(AppError::Internal),
        };
        Ok(Self {
            finding_id: record.finding_id,
            relation,
            public_title: record.public_title,
            currently_public: record.currently_public,
        })
    }
}
