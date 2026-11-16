use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::knowledge,
    enums::knowledge::KnowledgeEvidenceRelation,
    enums::knowledge_error::KnowledgeError,
    errors::app_error::AppError,
    models::knowledge::{KnowledgeCitationInputRecord, KnowledgeEntrySummary},
};

pub struct NewKnowledgeProposal<'a> {
    pub entry_id: Uuid,
    pub academic_subject_id: Uuid,
    pub title: &'a str,
    pub summary: &'a str,
    pub citations: &'a [KnowledgeCitationInputRecord<'a>],
    pub research_findings: &'a [(Uuid, KnowledgeEvidenceRelation)],
}

pub async fn create_knowledge_proposal(
    pool: &PgPool,
    contributor_account_id: Uuid,
    proposal: NewKnowledgeProposal<'_>,
) -> Result<KnowledgeEntrySummary, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let contributor_actor_id = find_verified_human_actor(&mut transaction, contributor_account_id)
        .await?
        .ok_or(KnowledgeError::ProposalInvalid)?;
    ensure_active_subject(&mut transaction, proposal.academic_subject_id).await?;
    ensure_findings_are_public(&mut transaction, proposal.research_findings).await?;
    insert_knowledge_entry(&mut transaction, contributor_actor_id, &proposal).await?;
    insert_knowledge_entry_revision(&mut transaction, contributor_actor_id, &proposal).await?;
    insert_proposal_citations(&mut transaction, contributor_actor_id, &proposal).await?;
    insert_proposal_research_findings(&mut transaction, &proposal).await?;
    let record = find_created_proposal(&mut transaction, proposal.entry_id).await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(record)
}

async fn find_verified_human_actor(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar(
        "SELECT identity.actor_id FROM human_actor_identities identity
        JOIN accounts account ON account.id = identity.account_id
        WHERE account.id = $1 AND account.account_status = 'active'
            AND account.email_verified_at IS NOT NULL FOR UPDATE OF account",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}

async fn ensure_active_subject(
    transaction: &mut Transaction<'_, Postgres>,
    subject_id: Uuid,
) -> Result<(), AppError> {
    let subject_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM academic_subjects WHERE id = $1 AND is_active)",
    )
    .bind(subject_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if subject_exists {
        Ok(())
    } else {
        Err(KnowledgeError::SubjectUnavailable.into())
    }
}

async fn ensure_findings_are_public(
    transaction: &mut Transaction<'_, Postgres>,
    findings: &[(Uuid, KnowledgeEvidenceRelation)],
) -> Result<(), AppError> {
    if findings.is_empty() {
        return Ok(());
    }
    let available_count: i64 = sqlx::query_scalar(
        "SELECT count(DISTINCT finding.id) FROM research_findings finding
        JOIN research_projects project ON project.id = finding.project_id
        WHERE finding.id = ANY($1) AND finding.finding_status = 'published'
            AND project.project_status = 'active' AND project.visibility = 'public'",
    )
    .bind(
        findings
            .iter()
            .map(|(finding_id, _)| *finding_id)
            .collect::<Vec<_>>(),
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if available_count == findings.len() as i64 {
        Ok(())
    } else {
        Err(KnowledgeError::ResearchFindingUnavailable.into())
    }
}

async fn insert_knowledge_entry(
    transaction: &mut Transaction<'_, Postgres>,
    contributor_actor_id: Uuid,
    proposal: &NewKnowledgeProposal<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO knowledge_entries
            (id, academic_subject_id, created_by_actor_id, entry_status, latest_revision_number)
        VALUES ($1, $2, $3, 'submitted', 1)",
    )
    .bind(proposal.entry_id)
    .bind(proposal.academic_subject_id)
    .bind(contributor_actor_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn insert_knowledge_entry_revision(
    transaction: &mut Transaction<'_, Postgres>,
    contributor_actor_id: Uuid,
    proposal: &NewKnowledgeProposal<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO knowledge_entry_revisions
            (entry_id, revision_number, title, summary, revision_status, authored_by_actor_id)
        VALUES ($1, 1, $2, $3, 'submitted', $4)",
    )
    .bind(proposal.entry_id)
    .bind(proposal.title)
    .bind(proposal.summary)
    .bind(contributor_actor_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn insert_proposal_citations(
    transaction: &mut Transaction<'_, Postgres>,
    contributor_actor_id: Uuid,
    proposal: &NewKnowledgeProposal<'_>,
) -> Result<(), AppError> {
    for citation in proposal.citations {
        let citation_id = insert_citation(transaction, contributor_actor_id, citation).await?;
        sqlx::query(
            "INSERT INTO knowledge_revision_citations
                (entry_id, revision_number, citation_id) VALUES ($1, 1, $2)",
        )
        .bind(proposal.entry_id)
        .bind(citation_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}

async fn insert_citation(
    transaction: &mut Transaction<'_, Postgres>,
    contributor_actor_id: Uuid,
    citation: &KnowledgeCitationInputRecord<'_>,
) -> Result<Uuid, AppError> {
    let new_id = Uuid::new_v4();
    let authors = serde_json::to_value(citation.authors).map_err(|_| AppError::Internal)?;
    sqlx::query_scalar(
        "INSERT INTO knowledge_citations
            (id, title, doi, external_url, authors, publication_year, license,
             provenance, created_by_actor_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id",
    )
    .bind(new_id)
    .bind(citation.title)
    .bind(citation.doi)
    .bind(citation.external_url)
    .bind(authors)
    .bind(citation.publication_year)
    .bind(citation.license)
    .bind(knowledge::KNOWLEDGE_CITATION_PROVENANCE)
    .bind(contributor_actor_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}

async fn insert_proposal_research_findings(
    transaction: &mut Transaction<'_, Postgres>,
    proposal: &NewKnowledgeProposal<'_>,
) -> Result<(), AppError> {
    for (finding_id, relation) in proposal.research_findings {
        sqlx::query(
            "INSERT INTO knowledge_revision_research_findings
                (entry_id, revision_number, finding_id, relation)
            VALUES ($1, 1, $2, $3)",
        )
        .bind(proposal.entry_id)
        .bind(finding_id)
        .bind(relation.as_database_value())
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}

async fn find_created_proposal(
    transaction: &mut Transaction<'_, Postgres>,
    entry_id: Uuid,
) -> Result<KnowledgeEntrySummary, AppError> {
    sqlx::query_as(
        "SELECT entry.id, entry.academic_subject_id, revision.title, revision.summary,
            revision.created_at
        FROM knowledge_entries entry
        JOIN knowledge_entry_revisions revision
            ON revision.entry_id = entry.id AND revision.revision_number = entry.latest_revision_number
        WHERE entry.id = $1 AND entry.entry_status = 'submitted'",
    )
    .bind(entry_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}
