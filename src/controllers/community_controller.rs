use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    constants::account_auth::SESSION_COOKIE_NAME,
    controllers::auth_cookie,
    dtos::{
        community_queries::ListCommunitiesQuery,
        community_requests::CreateCommunityRequest,
        community_responses::{CommunityMembershipResponse, CommunityResponse},
    },
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::{
        accounts::queries::account_session_queries,
        communities::{
            commands::{
                create_community_command::{self, CreateCommunityDetails},
                join_community_command, leave_community_command,
            },
            queries::find_community_query,
        },
    },
};

pub async fn list_communities(
    State(state): State<AppState>,
    Query(request): Query<ListCommunitiesQuery>,
) -> Result<Json<Vec<CommunityResponse>>, AppError> {
    let communities =
        find_community_query::list_discoverable_communities(&state.database, request.limit)
            .await?
            .into_iter()
            .map(CommunityResponse::from)
            .collect();
    Ok(Json(communities))
}

pub async fn create_community(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateCommunityRequest>,
) -> Result<impl IntoResponse, AppError> {
    let creator_account_id = require_account_id(&state, &headers).await?;
    let details = CreateCommunityDetails {
        slug: request.slug,
        display_name: request.display_name,
        description: request.description,
        category: request.category,
        visibility: request.visibility,
        membership_policy: request.membership_policy,
        primary_language_tag: request.primary_language_tag,
        institution_id: request.institution_id,
        course_id: request.course_id,
        discipline_id: request.discipline_id,
        subject_ids: request.subject_ids,
    };
    let community =
        create_community_command::create_community(&state.database, creator_account_id, details)
            .await?;
    Ok((
        StatusCode::CREATED,
        Json(CommunityResponse::from(community)),
    ))
}

pub async fn find_community(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_slug): Path<String>,
) -> Result<Json<CommunityResponse>, AppError> {
    let account_id = find_optional_account_id(&state, &headers).await?;
    let community = find_community_query::find_community_visible_to_account(
        &state.database,
        &community_slug,
        account_id,
    )
    .await?
    .ok_or(crate::enums::community_error::CommunityError::CommunityNotFound)?;
    Ok(Json(CommunityResponse::from(community)))
}

pub async fn join_community(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let membership =
        join_community_command::join_community(&state.database, &community_slug, account_id)
            .await?;
    let status = if membership.membership_status == "requested" {
        StatusCode::ACCEPTED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(CommunityMembershipResponse::from(membership))))
}

pub async fn leave_community(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    leave_community_command::leave_community(&state.database, &community_slug, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn require_account_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, AppError> {
    find_optional_account_id(state, headers)
        .await?
        .ok_or(AccountAuthError::SessionInvalid.into())
}

async fn find_optional_account_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<Uuid>, AppError> {
    let token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(token) = token else {
        return Ok(None);
    };
    let account = account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .ok_or(AccountAuthError::SessionInvalid)?;
    Ok(Some(account.id))
}
