use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::MEDIA_UPLOAD_SESSION_LIFETIME_SECONDS, errors::app_error::AppError,
    interfaces::media_storage::MediaStorage,
    repositories::communication_repository::media_repository,
};

pub async fn initiate_media_upload(
    pool: &PgPool,
    account_id: Uuid,
    resource_type: &str,
    resource_id: Option<Uuid>,
    original_filename: &str,
    media_type: &str,
    size_bytes: i64,
    checksum: &str,
) -> Result<(Uuid, Uuid, String, chrono::DateTime<chrono::Utc>), AppError> {
    media_repository::create_upload_session(
        pool,
        account_id,
        resource_type,
        resource_id,
        original_filename,
        media_type,
        size_bytes,
        checksum,
        MEDIA_UPLOAD_SESSION_LIFETIME_SECONDS,
    )
    .await
}

pub async fn upload_media_chunk(
    storage: &dyn MediaStorage,
    object_key: &str,
    data: &[u8],
) -> Result<(), AppError> {
    storage.write(object_key, data).await
}

pub async fn complete_media_upload(
    pool: &PgPool,
    account_id: Uuid,
    session_id: Uuid,
    checksum: &str,
) -> Result<Uuid, AppError> {
    media_repository::complete_upload(pool, account_id, session_id, checksum).await
}

pub async fn process_ready_media(pool: &PgPool, job_id: Uuid) -> Result<(), AppError> {
    media_repository::complete_processing_job(pool, job_id).await
}

pub async fn quarantine_media(pool: &PgPool, media_id: Uuid, reason: &str) -> Result<(), AppError> {
    media_repository::quarantine_media(pool, media_id, reason).await
}

pub async fn release_media(pool: &PgPool, media_id: Uuid) -> Result<(), AppError> {
    media_repository::release_media(pool, media_id).await
}

#[allow(dead_code)]
pub async fn delete_media(
    storage: &dyn MediaStorage,
    pool: &PgPool,
    media_id: Uuid,
) -> Result<(), AppError> {
    let object = media_repository::get_admin_media(pool, media_id).await?;
    storage.remove(&object.object_key).await
}
