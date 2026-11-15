use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::research_finding::ResearchSource,
    validations::research_finding_validation::ValidatedResearchSource,
};

use super::access;

pub async fn create_research_source(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    source_id: Uuid,
    source: ValidatedResearchSource,
) -> Result<ResearchSource, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    access::require_active_project(&mut transaction, project_id).await?;
    access::require_project_writer(&mut transaction, project_id, actor_id).await?;

    let insert_result = sqlx::query(
        "INSERT INTO research_sources (
            id, project_id, source_type, title, doi, source_url, authors,
            publication_year, provenance, license, created_by_actor_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(source_id)
    .bind(project_id)
    .bind(source.source_type.as_database_value())
    .bind(source.title)
    .bind(source.doi)
    .bind(source.source_url)
    .bind(source.authors)
    .bind(source.publication_year)
    .bind(source.provenance)
    .bind(source.license)
    .bind(actor_id)
    .execute(&mut *transaction)
    .await;
    if let Err(error) = insert_result {
        return Err(access::map_duplicate_doi(error));
    }
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    find_research_source_for_member(pool, source_id, account_id)
        .await?
        .ok_or(AppError::Internal)
}

pub async fn list_research_sources(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> Result<Vec<ResearchSource>, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    access::require_active_project(&mut transaction, project_id).await?;
    access::require_project_member(&mut transaction, project_id, actor_id).await?;
    let sources = query_as::<_, ResearchSource>(
        "SELECT source.id, source.project_id, source.source_type, source.title,
            source.doi, source.source_url, source.authors, source.publication_year,
            source.provenance, source.license, account.username AS created_by_username,
            source.created_at
        FROM research_sources source
        JOIN human_actor_identities identity ON identity.actor_id = source.created_by_actor_id
        JOIN accounts account ON account.id = identity.account_id
        WHERE source.project_id = $1
        ORDER BY source.created_at DESC, source.id LIMIT $2",
    )
    .bind(project_id)
    .bind(limit)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(sources)
}

async fn find_research_source_for_member(
    pool: &PgPool,
    source_id: Uuid,
    account_id: Uuid,
) -> Result<Option<ResearchSource>, AppError> {
    query_as::<_, ResearchSource>(
        "SELECT source.id, source.project_id, source.source_type, source.title,
            source.doi, source.source_url, source.authors, source.publication_year,
            source.provenance, source.license, account.username AS created_by_username,
            source.created_at
        FROM research_sources source
        JOIN human_actor_identities identity ON identity.actor_id = source.created_by_actor_id
        JOIN accounts account ON account.id = identity.account_id
        JOIN research_project_members member ON member.actor_id = (
            SELECT actor_id FROM human_actor_identities WHERE account_id = $2
        ) AND member.project_id = source.project_id AND member.membership_status = 'active'
        JOIN research_projects project ON project.id = source.project_id
            AND project.project_status = 'active'
        WHERE source.id = $1",
    )
    .bind(source_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}
