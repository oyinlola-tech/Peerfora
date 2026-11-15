use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    constants::research::{
        DEFAULT_RESEARCH_SOURCE_LIST_LIMIT, MAX_RESEARCH_FINDING_LIST_LIMIT,
        MAX_RESEARCH_SOURCE_LIST_LIMIT,
    },
    controllers::research_auth,
    dtos::research_finding::{
        CreateResearchFindingRequest, CreateResearchSourceRequest, ListResearchFindingsQuery,
        ResearchFindingResponse, ResearchFindingRevisionResponse, ResearchSourceResponse,
        ReviewResearchFindingRequest, UpdateResearchFindingRequest,
    },
    errors::app_error::AppError,
    models::app_state::AppState,
    services::research::{
        commands::{
            create_research_finding_command, create_research_source_command,
            publish_research_finding_command, retract_research_finding_command,
            review_research_finding_command, submit_research_finding_for_review_command,
            update_research_finding_command,
        },
        queries::{
            find_research_finding_query, list_research_finding_revisions_query,
            list_research_findings_query, list_research_sources_query,
        },
    },
    validations::research_finding_validation::{
        validate_research_finding, validate_research_finding_review_notes, validate_research_source,
    },
};

pub async fn create_research_source(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<CreateResearchSourceRequest>,
) -> Result<(StatusCode, Json<ResearchSourceResponse>), AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let source = create_research_source_command::create_research_source(
        &state.database,
        project_id,
        account_id,
        Uuid::new_v4(),
        validate_research_source(request)?,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(source.into())))
}

pub async fn list_research_sources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListResearchFindingsQuery>,
) -> Result<Json<Vec<ResearchSourceResponse>>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let limit = query
        .limit
        .unwrap_or(DEFAULT_RESEARCH_SOURCE_LIST_LIMIT)
        .clamp(1, MAX_RESEARCH_SOURCE_LIST_LIMIT);
    let sources = list_research_sources_query::list_research_sources(
        &state.database,
        project_id,
        account_id,
        limit,
    )
    .await?
    .into_iter()
    .map(ResearchSourceResponse::from)
    .collect();
    Ok(Json(sources))
}

pub async fn create_research_finding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<CreateResearchFindingRequest>,
) -> Result<(StatusCode, Json<ResearchFindingResponse>), AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let details = validate_research_finding(
        &request.title,
        &request.statement,
        &request.details,
        &request.citations,
    )?;
    let finding_id = Uuid::new_v4();
    create_research_finding_command::create_research_finding(
        &state.database,
        project_id,
        account_id,
        finding_id,
        details,
    )
    .await?;
    let finding = find_research_finding_query::find_research_finding(
        &state.database,
        finding_id,
        Some(account_id),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(finding.into())))
}

pub async fn list_research_findings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListResearchFindingsQuery>,
) -> Result<Json<Vec<ResearchFindingResponse>>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let limit = query
        .limit
        .unwrap_or(crate::constants::research::DEFAULT_RESEARCH_FINDING_LIST_LIMIT)
        .clamp(1, MAX_RESEARCH_FINDING_LIST_LIMIT);
    let findings = list_research_findings_query::list_research_findings(
        &state.database,
        project_id,
        account_id,
        limit,
    )
    .await?
    .into_iter()
    .map(ResearchFindingResponse::from)
    .collect();
    Ok(Json(findings))
}

pub async fn find_research_finding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(finding_id): Path<Uuid>,
) -> Result<Json<ResearchFindingResponse>, AppError> {
    let account_id = research_auth::find_optional_account_id(&state, &headers).await?;
    let finding =
        find_research_finding_query::find_research_finding(&state.database, finding_id, account_id)
            .await?;
    Ok(Json(finding.into()))
}

pub async fn list_research_finding_revisions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(finding_id): Path<Uuid>,
) -> Result<Json<Vec<ResearchFindingRevisionResponse>>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let revisions = list_research_finding_revisions_query::list_research_finding_revisions(
        &state.database,
        finding_id,
        account_id,
    )
    .await?
    .into_iter()
    .map(ResearchFindingRevisionResponse::from)
    .collect();
    Ok(Json(revisions))
}

pub async fn update_research_finding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(finding_id): Path<Uuid>,
    Json(request): Json<UpdateResearchFindingRequest>,
) -> Result<Json<ResearchFindingResponse>, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let details = validate_research_finding(
        &request.title,
        &request.statement,
        &request.details,
        &request.citations,
    )?;
    update_research_finding_command::update_research_finding(
        &state.database,
        finding_id,
        account_id,
        details,
    )
    .await?;
    let finding = find_research_finding_query::find_research_finding(
        &state.database,
        finding_id,
        Some(account_id),
    )
    .await?;
    Ok(Json(finding.into()))
}

pub async fn submit_research_finding_for_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(finding_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    submit_research_finding_for_review_command::submit_research_finding_for_review(
        &state.database,
        finding_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn review_research_finding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(finding_id): Path<Uuid>,
    Json(request): Json<ReviewResearchFindingRequest>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    let notes = validate_research_finding_review_notes(request.notes.as_deref())?;
    review_research_finding_command::review_research_finding(
        &state.database,
        finding_id,
        account_id,
        request.decision,
        notes,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn publish_research_finding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(finding_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    publish_research_finding_command::publish_research_finding(
        &state.database,
        finding_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn retract_research_finding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(finding_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = research_auth::require_authenticated_account_id(&state, &headers).await?;
    retract_research_finding_command::retract_research_finding(
        &state.database,
        finding_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
