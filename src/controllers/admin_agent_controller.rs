use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use uuid::Uuid;

use crate::{
    constants::agents,
    dtos::agent_identity::{
        AdministrateAgentRequest, AgentReviewItem, AgentReviewListResponse, AgentReviewQuery,
        ReviewAgentRequest, UpdateAgentResearchQuotaRequest,
    },
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::agents::{
        commands::{
            administrate_agent_lifecycle_command, review_agent_registration_command,
            update_agent_research_quota_command,
        },
        queries::list_agents_for_administrative_review_query,
    },
    utils::request_context::get_request_correlation_id,
};
use axum::http::StatusCode;

pub async fn list_agents_for_review(
    State(state): State<AppState>,
    Query(query): Query<AgentReviewQuery>,
) -> Result<Json<AgentReviewListResponse>, AppError> {
    let limit = query
        .limit
        .unwrap_or(agents::AGENT_REVIEW_DEFAULT_PAGE_SIZE)
        .clamp(1, agents::AGENT_REVIEW_MAX_PAGE_SIZE);
    let offset = query
        .offset
        .unwrap_or_default()
        .min(agents::AGENT_REVIEW_MAX_OFFSET);
    let (records, total_count) =
        list_agents_for_administrative_review_query::list_agents_for_administrative_review(
            &state.database,
            query.status,
            limit,
            offset,
        )
        .await?;
    let items = records
        .into_iter()
        .map(AgentReviewItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(AgentReviewListResponse {
        items,
        total_count,
        limit,
        offset,
    }))
}

pub async fn review_agent_registration(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(agent_actor_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ReviewAgentRequest>,
) -> Result<StatusCode, AppError> {
    review_agent_registration_command::review_agent_registration(
        &state.database,
        agent_actor_id,
        identity.administrator_id,
        request.decision,
        &request.approved_capabilities,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn administrate_agent_lifecycle(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(agent_actor_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AdministrateAgentRequest>,
) -> Result<StatusCode, AppError> {
    administrate_agent_lifecycle_command::administrate_agent(
        &state.database,
        agent_actor_id,
        identity.administrator_id,
        request.action,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_agent_research_quota(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(agent_actor_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<UpdateAgentResearchQuotaRequest>,
) -> Result<StatusCode, AppError> {
    update_agent_research_quota_command::update_agent_research_quota(
        &state.database,
        agent_actor_id,
        identity.administrator_id,
        request.submissions_per_24_hours,
        &request.reason,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
