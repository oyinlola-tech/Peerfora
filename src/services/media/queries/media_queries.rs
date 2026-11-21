use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{
        MEDIA_DEFAULT_PAGE_SIZE, MEDIA_MAX_PAGE_OFFSET, MEDIA_MAX_PAGE_SIZE, MEDIA_MAX_UPLOAD_BYTES,
    },
    errors::app_error::AppError,
    models::communication::MediaObjectSummary,
    repositories::communication_repository::media_repository,
};

pub async fn list_media(
    pool: &PgPool,
    account_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(Vec<MediaObjectSummary>, i64), AppError> {
    let limit = limit
        .unwrap_or(MEDIA_DEFAULT_PAGE_SIZE)
        .clamp(1, MEDIA_MAX_PAGE_SIZE);
    let offset = offset.unwrap_or(0).clamp(0, MEDIA_MAX_PAGE_OFFSET);
    media_repository::list_media_for_account(pool, account_id, limit, offset).await
}

pub async fn get_media(
    pool: &PgPool,
    account_id: Uuid,
    media_id: Uuid,
) -> Result<MediaObjectSummary, AppError> {
    media_repository::get_media_for_account(pool, account_id, media_id).await
}

pub fn validate_upload_limit(
    size_bytes: i64,
) -> Result<(), crate::enums::phase_four::CommunicationError> {
    if !(1..=MEDIA_MAX_UPLOAD_BYTES).contains(&size_bytes) {
        return Err(crate::enums::phase_four::CommunicationError::MediaInvalid);
    }
    Ok(())
}
