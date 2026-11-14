use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dtos::admin_auth::{
        AdministrativeCommunitiesResponse, AdministrativeCommunityItem,
        AdministrativeCommunityQuery, CommunityEnforcementRequest,
    },
    errors::app_error::AppError,
    models::{admin_identity::AdministratorSessionIdentity, app_state::AppState},
    services::{
        administration::queries::search_communities_for_administration_query::{
            CommunityAdministrationFilters, find_community_for_administration,
            search_communities_for_administration,
        },
        communities::commands::{
            restore_community_command::restore_community,
            suspend_community_command::suspend_community,
        },
    },
    utils::request_context::get_request_correlation_id,
};

pub async fn list_communities(
    State(state): State<AppState>,
    Query(query): Query<AdministrativeCommunityQuery>,
) -> Result<impl IntoResponse, AppError> {
    let filters = CommunityAdministrationFilters {
        search_prefix: query.search,
        category: query.category,
        lifecycle_status: query.status,
        limit: query.limit,
        offset: query.offset,
    };
    let (records, limit, offset, total_count) =
        search_communities_for_administration(&state.database, filters).await?;
    let items = records
        .into_iter()
        .map(AdministrativeCommunityItem::from)
        .collect();
    Ok(Json(AdministrativeCommunitiesResponse {
        items,
        total_count,
        limit,
        offset,
    }))
}

pub async fn show_community(
    State(state): State<AppState>,
    Path(community_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let record = find_community_for_administration(&state.database, community_id).await?;
    Ok(Json(AdministrativeCommunityItem::from(record)))
}

pub async fn suspend_community_access(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(community_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CommunityEnforcementRequest>,
) -> Result<impl IntoResponse, AppError> {
    suspend_community(
        &state.database,
        community_id,
        identity.administrator_id,
        &request.reason_code,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn restore_community_access(
    State(state): State<AppState>,
    Extension(identity): Extension<AdministratorSessionIdentity>,
    Path(community_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CommunityEnforcementRequest>,
) -> Result<impl IntoResponse, AppError> {
    restore_community(
        &state.database,
        community_id,
        identity.administrator_id,
        &request.reason_code,
        get_request_correlation_id(&headers),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

impl From<crate::models::admin_identity::AdministrativeCommunityRecord>
    for AdministrativeCommunityItem
{
    fn from(record: crate::models::admin_identity::AdministrativeCommunityRecord) -> Self {
        Self {
            id: record.id,
            slug: record.slug,
            display_name: record.display_name,
            category: record.category,
            visibility: record.visibility,
            lifecycle_status: record.lifecycle_status,
            owner_account_id: record.owner_account_id,
            member_count: record.member_count,
            created_at: record.created_at,
        }
    }
}
