use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    dtos::knowledge::{
        KnowledgeEntryPublishRequest, KnowledgeProposalReviewRequest,
        KnowledgeProposalReviewResponse, ListKnowledgeProposalsQuery,
    },
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::knowledge::{
        commands::{publish_knowledge_entry_command, review_knowledge_proposal_command},
        queries::{
            find_knowledge_proposal_for_administration_query,
            list_knowledge_proposals_for_administration_query,
        },
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn list_knowledge_proposals(
    State(state): State<AppState>,
    Query(query): Query<ListKnowledgeProposalsQuery>,
) -> Result<
    Json<list_knowledge_proposals_for_administration_query::KnowledgeProposalsPageResponse>,
    AppError,
> {
    let proposals = list_knowledge_proposals_for_administration_query::list_knowledge_proposals_for_administration(
        &state.database,
        query.limit,
        query.offset,
    )
    .await?;
    Ok(Json(proposals))
}

pub async fn find_knowledge_proposal(
    State(state): State<AppState>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<KnowledgeProposalReviewResponse>, AppError> {
    let proposal = find_knowledge_proposal_for_administration_query::find_knowledge_proposal_for_administration(
        &state.database,
        entry_id,
    )
    .await?;
    Ok(Json(proposal))
}

pub async fn review_knowledge_proposal(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(entry_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<KnowledgeProposalReviewRequest>,
) -> Result<StatusCode, AppError> {
    review_knowledge_proposal_command::review_proposal(
        &state.database,
        entry_id,
        identity.administrator_id,
        request.decision,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn publish_knowledge_entry(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(entry_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<KnowledgeEntryPublishRequest>,
) -> Result<StatusCode, AppError> {
    publish_knowledge_entry_command::publish_knowledge_entry(
        &state.database,
        entry_id,
        identity.administrator_id,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
