use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    controllers::research_auth,
    dtos::knowledge::{
        CreateKnowledgeProposalRequest, KnowledgeEntryResponse, ListKnowledgeEntriesQuery,
    },
    errors::app_error::AppError,
    models::app_state::AppState,
    services::knowledge::{
        commands::create_knowledge_proposal_command,
        queries::{find_public_knowledge_entry_query, list_knowledge_entries_query},
    },
};

pub async fn submit_knowledge_proposal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateKnowledgeProposalRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::dtos::knowledge::KnowledgeProposalCreatedResponse>,
    ),
    AppError,
> {
    let contributor_account_id =
        research_auth::require_authenticated_account_id(&state, &headers).await?;
    let proposal = create_knowledge_proposal_command::submit_knowledge_proposal(
        &state.database,
        contributor_account_id,
        request,
    )
    .await?;
    Ok((StatusCode::ACCEPTED, Json(proposal)))
}

pub async fn list_public_knowledge_entries(
    State(state): State<AppState>,
    Query(query): Query<ListKnowledgeEntriesQuery>,
) -> Result<Json<list_knowledge_entries_query::KnowledgeEntriesPageResponse>, AppError> {
    let entries =
        list_knowledge_entries_query::list_knowledge_entries(&state.database, query).await?;
    Ok(Json(entries))
}

pub async fn find_public_knowledge_entry(
    State(state): State<AppState>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<KnowledgeEntryResponse>, AppError> {
    let entry = find_public_knowledge_entry_query::find_knowledge_entry(&state.database, entry_id)
        .await?
        .ok_or(crate::enums::knowledge_error::KnowledgeError::ProposalNotFound)?;
    Ok(Json(entry))
}
