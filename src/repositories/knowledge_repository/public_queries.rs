use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    models::knowledge::{
        KnowledgeCitationRecord, KnowledgeEntryRecord, KnowledgeEntrySummary,
        KnowledgeFindingReferenceRecord,
    },
};

pub async fn list_public_knowledge_entries(
    pool: &PgPool,
    query: Option<&str>,
    academic_subject_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<KnowledgeEntrySummary>, i64), AppError> {
    let total_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM knowledge_entries entry
        JOIN knowledge_entry_revisions revision
            ON revision.entry_id = entry.id
            AND revision.revision_number = entry.published_revision_number
        JOIN academic_subjects subject ON subject.id = entry.academic_subject_id
            AND subject.is_active
        WHERE entry.entry_status = 'published'
            AND revision.revision_status = 'published'
            AND ($1::text IS NULL OR revision.search_document @@ websearch_to_tsquery('simple', $1))
            AND ($2::uuid IS NULL OR entry.academic_subject_id = $2)",
    )
    .bind(query)
    .bind(academic_subject_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let entries = sqlx::query_as(
        "SELECT entry.id, entry.academic_subject_id, revision.title, revision.summary,
            revision.created_at
        FROM knowledge_entries entry
        JOIN knowledge_entry_revisions revision
            ON revision.entry_id = entry.id
            AND revision.revision_number = entry.published_revision_number
        JOIN academic_subjects subject ON subject.id = entry.academic_subject_id
            AND subject.is_active
        WHERE entry.entry_status = 'published'
            AND revision.revision_status = 'published'
            AND ($1::text IS NULL OR revision.search_document @@ websearch_to_tsquery('simple', $1))
            AND ($2::uuid IS NULL OR entry.academic_subject_id = $2)
        ORDER BY entry.updated_at DESC, entry.id
        LIMIT $3 OFFSET $4",
    )
    .bind(query)
    .bind(academic_subject_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((entries, total_count))
}

pub async fn find_public_knowledge_entry(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Option<KnowledgeEntryRecord>, AppError> {
    sqlx::query_as(
        "SELECT entry.id, entry.academic_subject_id, revision.title, revision.summary,
            revision.created_at
        FROM knowledge_entries entry
        JOIN knowledge_entry_revisions revision
            ON revision.entry_id = entry.id
            AND revision.revision_number = entry.published_revision_number
        JOIN academic_subjects subject ON subject.id = entry.academic_subject_id
            AND subject.is_active
        WHERE entry.id = $1 AND entry.entry_status = 'published'
            AND revision.revision_status = 'published'",
    )
    .bind(entry_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_entry_citations(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Vec<KnowledgeCitationRecord>, AppError> {
    sqlx::query_as(
        "SELECT citation.title, citation.doi, citation.external_url, citation.authors,
            citation.publication_year, citation.license
        FROM knowledge_revision_citations link
        JOIN knowledge_entries entry ON entry.id = link.entry_id
            AND entry.published_revision_number = link.revision_number
        JOIN knowledge_citations citation ON citation.id = link.citation_id
        WHERE entry.id = $1 AND entry.entry_status = 'published'
        ORDER BY citation.title, citation.id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_public_entry_research_findings(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Vec<KnowledgeFindingReferenceRecord>, AppError> {
    sqlx::query_as(
        "SELECT finding.id AS finding_id, link.relation, revision.title
        FROM knowledge_revision_research_findings link
        JOIN knowledge_entries entry ON entry.id = link.entry_id
            AND entry.published_revision_number = link.revision_number
        JOIN research_findings finding ON finding.id = link.finding_id
            AND finding.finding_status = 'published'
        JOIN research_finding_revisions revision
            ON revision.finding_id = finding.id
            AND revision.revision_number = finding.published_revision_number
        JOIN research_projects project ON project.id = finding.project_id
            AND project.project_status = 'active' AND project.visibility = 'public'
        WHERE entry.id = $1 AND entry.entry_status = 'published'
        ORDER BY finding.id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
