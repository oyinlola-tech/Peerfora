use sqlx::PgPool;

use crate::{
    constants::knowledge, dtos::knowledge::KnowledgeEntrySummaryResponse,
    errors::app_error::AppError,
    repositories::knowledge_repository::administration::list_submitted_knowledge_proposals,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeProposalsPageResponse {
    pub items: Vec<KnowledgeEntrySummaryResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_knowledge_proposals_for_administration(
    pool: &PgPool,
    requested_limit: Option<i64>,
    requested_offset: Option<i64>,
) -> Result<KnowledgeProposalsPageResponse, AppError> {
    let limit = requested_limit
        .unwrap_or(knowledge::KNOWLEDGE_DEFAULT_PAGE_SIZE)
        .clamp(1, knowledge::KNOWLEDGE_MAX_PAGE_SIZE);
    let offset = requested_offset
        .unwrap_or_default()
        .clamp(0, knowledge::KNOWLEDGE_MAX_OFFSET);
    let (items, total_count) = list_submitted_knowledge_proposals(pool, limit, offset).await?;
    Ok(KnowledgeProposalsPageResponse {
        items: items.into_iter().map(Into::into).collect(),
        total_count,
        limit,
        offset,
    })
}
