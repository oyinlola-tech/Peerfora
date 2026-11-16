use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dtos::knowledge::KnowledgeEntryResponse,
    errors::app_error::AppError,
    repositories::knowledge_repository::public_queries::{
        find_public_knowledge_entry, list_entry_citations, list_public_entry_research_findings,
    },
};

pub async fn find_knowledge_entry(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Option<KnowledgeEntryResponse>, AppError> {
    let Some(entry) = find_public_knowledge_entry(pool, entry_id).await? else {
        return Ok(None);
    };
    let citations = list_entry_citations(pool, entry_id).await?;
    let findings = list_public_entry_research_findings(pool, entry_id).await?;
    KnowledgeEntryResponse::try_from((entry, citations, findings)).map(Some)
}
