use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::{
    constants::{
        account_auth::SESSION_COOKIE_NAME,
        discussions::{
            MAX_COMMUNITY_POSTS_LIMIT, MAX_POST_COMMENTS_LIMIT, MAX_POST_COMMENTS_OFFSET,
        },
    },
    controllers::auth_cookie,
    dtos::discussion::{
        CommunityPostResponse, CreateCommunityPostRequest, CreatePostCommentRequest,
        CreatePostResearchFindingReferenceRequest, ListCommunityPostsQuery, ListPostCommentsQuery,
        PostCommentResponse,
    },
    dtos::research_finding::ResearchFindingResponse,
    enums::account_auth_error::AccountAuthError,
    errors::app_error::AppError,
    models::app_state::AppState,
    services::{
        accounts::queries::account_session_queries,
        discussions::{
            commands::{
                create_community_post_command, create_post_comment_command,
                link_public_research_finding_to_post_command,
                remove_post_research_finding_reference_command,
            },
            queries::{
                list_community_posts_query, list_post_comments_query,
                list_post_research_finding_references_query,
            },
        },
    },
};

pub async fn create_community_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_slug): Path<String>,
    Json(request): Json<CreateCommunityPostRequest>,
) -> Result<(StatusCode, Json<CommunityPostResponse>), AppError> {
    let author_account_id = require_account_id(&state, &headers).await?;
    let post = create_community_post_command::create_community_post(
        &state.database,
        &community_slug,
        author_account_id,
        request.kind,
        &request.title,
        &request.body,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(CommunityPostResponse::from(post))))
}

pub async fn list_community_posts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(community_slug): Path<String>,
    Query(request): Query<ListCommunityPostsQuery>,
) -> Result<Json<Vec<CommunityPostResponse>>, AppError> {
    let viewer_account_id = find_optional_account_id(&state, &headers).await?;
    let posts = list_community_posts_query::list_community_posts(
        &state.database,
        &community_slug,
        viewer_account_id,
        request
            .limit
            .map(|limit| limit.clamp(1, MAX_COMMUNITY_POSTS_LIMIT)),
    )
    .await?
    .into_iter()
    .map(CommunityPostResponse::from)
    .collect();
    Ok(Json(posts))
}

pub async fn create_post_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(post_id): Path<Uuid>,
    Json(request): Json<CreatePostCommentRequest>,
) -> Result<(StatusCode, Json<PostCommentResponse>), AppError> {
    let author_account_id = require_account_id(&state, &headers).await?;
    let comment = create_post_comment_command::create_post_comment(
        &state.database,
        post_id,
        author_account_id,
        request.parent_comment_id,
        &request.body,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(PostCommentResponse::from(comment)),
    ))
}

pub async fn list_post_comments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(post_id): Path<Uuid>,
    Query(request): Query<ListPostCommentsQuery>,
) -> Result<Json<Vec<PostCommentResponse>>, AppError> {
    let viewer_account_id = find_optional_account_id(&state, &headers).await?;
    let comments = list_post_comments_query::list_post_comments(
        &state.database,
        post_id,
        viewer_account_id,
        request
            .limit
            .map(|limit| limit.clamp(1, MAX_POST_COMMENTS_LIMIT)),
        request
            .offset
            .map(|offset| offset.clamp(0, MAX_POST_COMMENTS_OFFSET)),
    )
    .await?
    .into_iter()
    .map(PostCommentResponse::from)
    .collect();
    Ok(Json(comments))
}

pub async fn link_public_research_finding_to_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(post_id): Path<Uuid>,
    Json(request): Json<CreatePostResearchFindingReferenceRequest>,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    link_public_research_finding_to_post_command::link_public_research_finding_to_post(
        &state.database,
        post_id,
        request.finding_id,
        account_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_post_research_finding_references(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(post_id): Path<Uuid>,
) -> Result<Json<Vec<ResearchFindingResponse>>, AppError> {
    let viewer_account_id = find_optional_account_id(&state, &headers).await?;
    let findings =
        list_post_research_finding_references_query::list_post_research_finding_references(
            &state.database,
            post_id,
            viewer_account_id,
        )
        .await?
        .into_iter()
        .map(ResearchFindingResponse::from)
        .collect();
    Ok(Json(findings))
}

pub async fn remove_post_research_finding_reference(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((post_id, finding_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    remove_post_research_finding_reference_command::remove_post_research_finding_reference(
        &state.database,
        post_id,
        finding_id,
        account_id,
    )
    .await?;
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
    let session_token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(session_token) = session_token else {
        return Ok(None);
    };
    let account =
        account_session_queries::find_account_for_session(&state.database, &session_token)
            .await?
            .ok_or(AccountAuthError::SessionInvalid)?;
    Ok(Some(account.id))
}
