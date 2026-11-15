use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::research::ResearchFindingError, errors::app_error::AppError,
    validations::research_finding_validation::ValidatedResearchFinding,
};

pub async fn ensure_citations_belong_to_project(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    details: &ValidatedResearchFinding,
) -> Result<(), AppError> {
    let source_ids = details
        .citations
        .iter()
        .map(|citation| citation.source_id)
        .collect::<Vec<_>>();
    let source_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM research_sources
        WHERE project_id = $1 AND id = ANY($2)",
    )
    .bind(project_id)
    .bind(&source_ids)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if source_count == source_ids.len() as i64 {
        Ok(())
    } else {
        Err(ResearchFindingError::CitationNotInProject.into())
    }
}

pub async fn insert_finding_revision(
    transaction: &mut Transaction<'_, Postgres>,
    finding_id: Uuid,
    project_id: Uuid,
    revision_number: i16,
    author_actor_id: Uuid,
    details: &ValidatedResearchFinding,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO research_finding_revisions (
            finding_id, project_id, revision_number, title, statement, details,
            authored_by_actor_id, review_status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'draft')",
    )
    .bind(finding_id)
    .bind(project_id)
    .bind(revision_number)
    .bind(&details.title)
    .bind(&details.statement)
    .bind(&details.details)
    .bind(author_actor_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    for citation in &details.citations {
        sqlx::query(
            "INSERT INTO research_finding_revision_sources (
                finding_id, project_id, revision_number, source_id, citation_locator,
                citation_context, added_by_actor_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(finding_id)
        .bind(project_id)
        .bind(revision_number)
        .bind(citation.source_id)
        .bind(&citation.locator)
        .bind(&citation.context)
        .bind(author_actor_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}
