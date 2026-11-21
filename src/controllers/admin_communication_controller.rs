use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dtos::communication::MediaObjectResponse,
    enums::communication::MediaReviewAction,
    enums::phase_four::CommunicationError,
    errors::app_error::AppError,
    models::app_state::AppState,
    repositories::communication_repository::{conversation_repository, media_repository},
    services::media::commands::media_commands,
};

#[derive(Deserialize)]
pub struct MediaReviewInput {
    pub action: String,
    pub reason: String,
}

pub async fn review_media(
    State(state): State<AppState>,
    Path(media_id): Path<Uuid>,
    Json(request): Json<MediaReviewInput>,
) -> Result<Json<MediaObjectResponse>, AppError> {
    let media = media_repository::get_admin_media(&state.database, media_id).await?;
    match MediaReviewAction::from_input(&request.action) {
        Some(MediaReviewAction::Quarantine) if !request.reason.trim().is_empty() => {
            media_commands::quarantine_media(&state.database, media.id, request.reason.trim())
                .await?;
        }
        Some(MediaReviewAction::Release) => {
            media_commands::release_media(&state.database, media.id).await?
        }
        _ => {
            return Err(CommunicationError::MediaInvalid.into());
        }
    }
    let media = media_repository::get_admin_media(&state.database, media_id).await?;
    Ok(Json(MediaObjectResponse::from(media)))
}

pub async fn restrict_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    conversation_repository::set_lifecycle_status(
        &state.database,
        conversation_id,
        crate::models::communication::ConversationLifecycleStatus::Restricted,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn close_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    conversation_repository::set_lifecycle_status(
        &state.database,
        conversation_id,
        crate::models::communication::ConversationLifecycleStatus::Closed,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
