use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    dtos::admin_research::{
        AdministrativeResearchProjectItem, AdministrativeResearchProjectQuery,
        AdministrativeResearchProjectsResponse, ResearchProjectEnforcementRequest,
    },
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::{
        administration::queries::{
            find_research_project_for_administration_query,
            search_research_projects_for_administration_query::{
                ResearchProjectAdministrationFilters, search_research_projects_for_administration,
            },
        },
        research::commands::{restore_research_project_command, suspend_research_project_command},
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn list_research_projects(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Query(query): Query<AdministrativeResearchProjectQuery>,
    headers: HeaderMap,
) -> Result<Json<AdministrativeResearchProjectsResponse>, AppError> {
    let filters = ResearchProjectAdministrationFilters {
        search_prefix: query.search,
        status: query.status,
        limit: query.limit,
        offset: query.offset,
    };
    let (records, limit, offset, total_count) = search_research_projects_for_administration(
        &state.database,
        identity.administrator_id,
        filters,
        get_request_correlation_id(&headers),
    )
    .await?;
    let items = records
        .into_iter()
        .map(AdministrativeResearchProjectItem::from)
        .collect();
    Ok(Json(AdministrativeResearchProjectsResponse {
        items,
        total_count,
        limit,
        offset,
    }))
}

pub async fn show_research_project(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<AdministrativeResearchProjectItem>, AppError> {
    let record =
        find_research_project_for_administration_query::find_research_project_for_administration(
            &state.database,
            project_id,
            identity.administrator_id,
            get_request_correlation_id(&headers),
        )
        .await?;
    Ok(Json(AdministrativeResearchProjectItem::from(record)))
}

pub async fn suspend_research_project(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ResearchProjectEnforcementRequest>,
) -> Result<StatusCode, AppError> {
    suspend_research_project_command::suspend_research_project(
        &state.database,
        project_id,
        identity.administrator_id,
        &request.reason_code,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_research_project(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ResearchProjectEnforcementRequest>,
) -> Result<StatusCode, AppError> {
    restore_research_project_command::restore_research_project(
        &state.database,
        project_id,
        identity.administrator_id,
        &request.reason_code,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
