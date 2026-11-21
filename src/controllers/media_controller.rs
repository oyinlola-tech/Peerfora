use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    constants::{account_auth::SESSION_COOKIE_NAME, phase_four::MEDIA_UPLOADS_ROUTE},
    controllers::auth_cookie,
    dtos::communication::{MediaObjectResponse, MediaUploadInitiationResponse},
    enums::{
        account_auth_error::AccountAuthError, communication::MediaProcessingStatus,
        phase_four::CommunicationError,
    },
    errors::app_error::AppError,
    models::{
        app_state::AppState,
        communication::{MediaReportInput, MediaUploadCompleteInput, MediaUploadInitInput},
    },
    repositories::communication_repository::media_repository,
    services::moderation::commands::create_moderation_report_command as moderation_report_commands,
    services::{
        accounts::queries::account_session_queries,
        media::{commands::media_commands, queries::media_queries},
    },
};

#[derive(Deserialize)]
pub struct MediaListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn initiate_media_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<MediaUploadInitInput>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    media_queries::validate_upload_limit(request.size_bytes)?;
    let (media_object_id, upload_session_id, upload_token, expires_at) =
        media_commands::initiate_media_upload(
            &state.database,
            account_id,
            &request.resource_type,
            request.resource_id,
            &request.original_filename,
            &request.media_type,
            request.size_bytes,
            &request.checksum,
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(MediaUploadInitiationResponse {
            media_object_id,
            upload_session_id,
            upload_token,
            upload_url: format!("{MEDIA_UPLOADS_ROUTE}/{upload_session_id}"),
            expires_at,
        }),
    ))
}

pub async fn upload_media_bytes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<Uuid>,
    body: axum::body::Bytes,
) -> Result<StatusCode, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    if body.is_empty() {
        return Err(CommunicationError::MediaInvalid.into());
    }
    let session = crate::repositories::communication_repository::media_repository::get_upload_session_for_owner(
        &state.database,
        account_id,
        session_id,
    )
    .await?;
    media_commands::upload_media_chunk(&*state.media_storage, &session.object_key, &body).await
}

pub async fn complete_media_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<Uuid>,
    Json(request): Json<MediaUploadCompleteInput>,
) -> Result<Json<MediaObjectResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let _session =
        media_repository::get_upload_session_for_owner(&state.database, account_id, session_id)
            .await?;
    let media_object_id = media_commands::complete_media_upload(
        &state.database,
        account_id,
        session_id,
        &request.checksum,
    )
    .await?;
    let media = media_queries::get_media(&state.database, account_id, media_object_id).await?;
    Ok(Json(MediaObjectResponse::from(media)))
}

pub async fn list_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<MediaListQuery>,
) -> Result<Json<Vec<MediaObjectResponse>>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let (media, _) =
        media_queries::list_media(&state.database, account_id, query.limit, query.offset).await?;
    Ok(Json(
        media.into_iter().map(MediaObjectResponse::from).collect(),
    ))
}

pub async fn get_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(media_id): Path<Uuid>,
) -> Result<Json<MediaObjectResponse>, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let media = media_queries::get_media(&state.database, account_id, media_id).await?;
    Ok(Json(MediaObjectResponse::from(media)))
}

pub async fn download_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(media_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let media = media_queries::get_media(&state.database, account_id, media_id).await?;
    let processing_status = MediaProcessingStatus::from_db_value(&media.processing_status)
        .ok_or(CommunicationError::MediaNotReady)?;
    if !processing_status.permits_download() {
        return Err(CommunicationError::MediaNotReady.into());
    }
    let data = state.media_storage.read(&media.object_key).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, media.media_type.clone())],
        [(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", media.original_filename),
        )],
        data,
    ))
}

pub async fn report_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(media_id): Path<Uuid>,
    Json(request): Json<MediaReportInput>,
) -> Result<impl IntoResponse, AppError> {
    let account_id = require_account_id(&state, &headers).await?;
    let media = media_queries::get_media(&state.database, account_id, media_id).await?;
    moderation_report_commands::create_moderation_report(
        &state.database,
        account_id,
        crate::models::moderation::ModerationReportTarget::Media(media.id),
        crate::enums::moderation::ModerationReportCategory::from_category(&request.category)?,
        &request.description,
    )
    .await?;
    Ok(StatusCode::CREATED)
}

async fn require_account_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, AppError> {
    let token = auth_cookie::read_bearer_token(headers)
        .or_else(|| auth_cookie::read_auth_cookie(headers, SESSION_COOKIE_NAME));
    let Some(token) = token else {
        return Err(AccountAuthError::SessionInvalid.into());
    };
    let account = account_session_queries::find_account_for_session(&state.database, &token)
        .await?
        .ok_or(AccountAuthError::SessionInvalid)?;
    Ok(account.id)
}
