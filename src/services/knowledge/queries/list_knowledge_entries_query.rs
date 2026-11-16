use crate::{
    constants::knowledge,
    dtos::knowledge::{KnowledgeEntrySummaryResponse, ListKnowledgeEntriesQuery},
    enums::knowledge_error::KnowledgeError,
    errors::app_error::AppError,
    repositories::knowledge_repository::public_queries::list_public_knowledge_entries,
};
use sqlx::PgPool;

pub async fn list_knowledge_entries(
    pool: &PgPool,
    query: ListKnowledgeEntriesQuery,
) -> Result<KnowledgeEntriesPageResponse, AppError> {
    let query_text = validate_search_query(query.query)?;
    let limit = query
        .limit
        .unwrap_or(knowledge::KNOWLEDGE_DEFAULT_PAGE_SIZE)
        .clamp(1, knowledge::KNOWLEDGE_MAX_PAGE_SIZE);
    let offset = query
        .offset
        .unwrap_or_default()
        .clamp(0, knowledge::KNOWLEDGE_MAX_OFFSET);
    let (entries, total_count) = list_public_knowledge_entries(
        pool,
        query_text.as_deref(),
        query.academic_subject_id,
        limit,
        offset,
    )
    .await?;
    Ok(KnowledgeEntriesPageResponse {
        items: entries.into_iter().map(Into::into).collect(),
        total_count,
        limit,
        offset,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEntriesPageResponse {
    pub items: Vec<KnowledgeEntrySummaryResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

fn validate_search_query(query: Option<String>) -> Result<Option<String>, AppError> {
    query
        .map(|query| {
            let query = query.trim();
            if query.is_empty() {
                return Ok(None);
            }
            if query.chars().count() > knowledge::KNOWLEDGE_SEARCH_QUERY_MAX_LENGTH {
                return Err(KnowledgeError::ProposalInvalid.into());
            }
            Ok(Some(query.to_owned()))
        })
        .transpose()
        .map(Option::flatten)
}
