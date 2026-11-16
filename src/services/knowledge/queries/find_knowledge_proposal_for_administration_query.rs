use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dtos::knowledge::KnowledgeProposalReviewResponse, enums::knowledge_error::KnowledgeError,
    errors::app_error::AppError, repositories::knowledge_repository::administration,
};

pub async fn find_knowledge_proposal_for_administration(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<KnowledgeProposalReviewResponse, AppError> {
    let proposal = administration::find_submitted_knowledge_proposal(pool, entry_id)
        .await?
        .ok_or(KnowledgeError::ProposalNotFound)?;
    let citations = administration::list_submitted_proposal_citations(pool, entry_id).await?;
    let research_findings =
        administration::list_submitted_proposal_research_findings(pool, entry_id).await?;
    KnowledgeProposalReviewResponse::try_from((proposal, citations, research_findings))
}
