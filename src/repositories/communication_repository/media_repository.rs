use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{
        MEDIA_MAX_FILENAME_LENGTH, MEDIA_MAX_MEDIA_TYPE_LENGTH, MEDIA_MAX_PAGE_OFFSET,
        MEDIA_MAX_PAGE_SIZE, MEDIA_MAX_RESOURCE_TYPE_LENGTH, MEDIA_MAX_UPLOAD_BYTES,
        MEDIA_PROCESSING_CLAIM_SECONDS,
    },
    enums::account_lifecycle::AccountLifecycleStatus,
    enums::communication::{
        ConversationLifecycleStatus, MediaProcessingJobStatus, MediaProcessingStatus,
        MediaResourceType, MediaUploadSessionStatus, MembershipStatus,
    },
    enums::phase_four::CommunicationError,
    errors::app_error::AppError,
    models::communication::MediaObjectSummary,
};

pub fn validate_media_upload(
    resource_type: &str,
    original_filename: &str,
    media_type: &str,
    size_bytes: i64,
    checksum: &str,
) -> Result<(), CommunicationError> {
    if resource_type.is_empty()
        || resource_type.len() > MEDIA_MAX_RESOURCE_TYPE_LENGTH
        || original_filename.trim().is_empty()
        || original_filename.chars().count() > MEDIA_MAX_FILENAME_LENGTH
        || media_type.is_empty()
        || media_type.len() > MEDIA_MAX_MEDIA_TYPE_LENGTH
        || size_bytes <= 0
        || size_bytes > MEDIA_MAX_UPLOAD_BYTES
        || checksum.len() != 64
        || checksum
            .chars()
            .any(|character| !character.is_ascii_hexdigit())
    {
        return Err(CommunicationError::MediaInvalid);
    }
    Ok(())
}

pub fn hash_upload_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub async fn create_upload_session(
    pool: &PgPool,
    owner_account_id: Uuid,
    resource_type: &str,
    resource_id: Option<Uuid>,
    original_filename: &str,
    media_type: &str,
    size_bytes: i64,
    checksum: &str,
    lifetime_seconds: i64,
) -> Result<(Uuid, Uuid, String, DateTime<Utc>), AppError> {
    validate_media_upload(
        resource_type,
        original_filename,
        media_type,
        size_bytes,
        checksum,
    )?;
    let object_key = format!("{owner_account_id}/{}/{Uuid::new_v4()}", resource_type);
    let uploading = MediaProcessingStatus::Uploading.as_db_value();
    let media_object_id: Uuid = sqlx::query_scalar(
        "INSERT INTO media_objects (owner_account_id, resource_type, resource_id, object_key,\
            original_filename, media_type, size_bytes, checksum, processing_status)\
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
    )
    .bind(owner_account_id)
    .bind(resource_type)
    .bind(resource_id)
    .bind(object_key)
    .bind(original_filename)
    .bind(media_type)
    .bind(size_bytes)
    .bind(checksum.to_ascii_lowercase())
    .bind(uploading)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    let token = Uuid::new_v4().simple().to_string();
    let token_hash = hash_upload_token(&token);
    let session_uploading = MediaUploadSessionStatus::Uploading.as_db_value();
    let (session_id, expires_at): (Uuid, DateTime<Utc>) = sqlx::query_as(
        "INSERT INTO media_upload_sessions (media_object_id, upload_token_hash, status, expires_at)\
         VALUES ($1, $2, $3, now() + make_interval(secs => $4))\
         RETURNING id, expires_at",
    )
    .bind(media_object_id)
    .bind(&token_hash)
    .bind(session_uploading)
    .bind(lifetime_seconds)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((media_object_id, session_id, token, expires_at))
}

#[derive(sqlx::FromRow)]
pub struct MediaUploadSessionForOwner {
    pub session_id: Uuid,
    pub object_key: String,
}

pub async fn get_upload_session_for_owner(
    pool: &PgPool,
    owner_account_id: Uuid,
    session_id: Uuid,
) -> Result<MediaUploadSessionForOwner, AppError> {
    let session_uploading = MediaUploadSessionStatus::Uploading.as_db_value();
    sqlx::query_as::<_, MediaUploadSessionForOwner>(
        "SELECT media_upload_sessions.id AS session_id, object.object_key\
         FROM media_upload_sessions\
         JOIN media_objects object ON object.id = media_upload_sessions.media_object_id\
         WHERE media_upload_sessions.id = $1 AND object.owner_account_id = $2\
             AND media_upload_sessions.status = $3\
             AND media_upload_sessions.expires_at > now()",
    )
    .bind(session_id)
    .bind(owner_account_id)
    .bind(session_uploading)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::MediaUploadTokenInvalid.into())
}

pub async fn complete_upload(
    pool: &PgPool,
    owner_account_id: Uuid,
    session_id: Uuid,
    checksum: &str,
) -> Result<Uuid, AppError> {
    if checksum.len() != 64 || checksum.chars().any(|c| !c.is_ascii_hexdigit()) {
        return Err(CommunicationError::MediaInvalid.into());
    }
    let session_uploading = MediaUploadSessionStatus::Uploading.as_db_value();
    let media_uploaded = MediaProcessingStatus::Uploaded.as_db_value();
    let result = sqlx::query_scalar::<_, Uuid>(
        "WITH authorized AS (\
            SELECT media_upload_sessions.media_object_id FROM media_upload_sessions\
            JOIN media_objects object ON object.id = media_upload_sessions.media_object_id\
            WHERE media_upload_sessions.id = $1 AND object.owner_account_id = $2\
                AND media_upload_sessions.status = $3\
                AND media_upload_sessions.expires_at > now()\
                AND object.checksum = lower($4)\
        )\
        UPDATE media_objects SET processing_status = $5, updated_at = now()\
        WHERE id IN (SELECT media_object_id FROM authorized)\
        RETURNING id",
    )
    .bind(session_id)
    .bind(owner_account_id)
    .bind(session_uploading)
    .bind(checksum.to_ascii_lowercase())
    .bind(media_uploaded)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    result.ok_or(CommunicationError::MediaUploadTokenInvalid.into())
}

pub async fn get_media_for_account(
    pool: &PgPool,
    account_id: Uuid,
    media_id: Uuid,
) -> Result<MediaObjectSummary, AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let active_account = AccountLifecycleStatus::Active.as_database_value();
    let message_resource = MediaResourceType::Message.as_db_value();
    let project_resource = MediaResourceType::ResearchProject.as_db_value();
    sqlx::query_as::<_, MediaObjectSummary>(
        "SELECT object.id, object.owner_account_id, object.resource_type, object.resource_id,\
            object.object_key, object.original_filename, object.media_type, object.size_bytes,\
            object.checksum, object.processing_status, object.created_at, object.updated_at,\
            CASE WHEN object.resource_id IS NOT NULL THEN object.resource_id::TEXT ELSE NULL END AS authorization_resource\
         FROM media_objects object\
         WHERE object.id = $1\
             AND (object.owner_account_id = $2\
                 OR (object.resource_id IS NOT NULL AND object.resource_type = $3\
                     AND EXISTS (\
                         SELECT 1 FROM messages message\
                         JOIN conversation_members membership ON membership.conversation_id = message.conversation_id\
                             AND membership.account_id = $2 AND membership.membership_status = $4\
                         WHERE message.id = object.resource_id\
                     ))\
                 OR (object.resource_id IS NOT NULL AND object.resource_type = $5\
                     AND EXISTS (\
                         SELECT 1 FROM research_projects project\
                         JOIN human_actor_identities actor ON actor.account_id = $2\
                         WHERE project.id = object.resource_id AND project.project_status = $6\
                             AND actor.actor_status = $7\
                             AND EXISTS (\
                                 SELECT 1 FROM research_project_members member\
                                 WHERE member.project_id = project.id AND member.actor_id = actor.actor_id\
                                     AND member.membership_status = $4\
                             ))\
                 ))\
         FOR UPDATE OF object",
    )
    .bind(media_id)
    .bind(account_id)
    .bind(message_resource)
    .bind(active_membership)
    .bind(project_resource)
    .bind(active_lifecycle)
    .bind(active_account)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::MediaNotFound.into())
}

pub async fn list_media_for_account(
    pool: &PgPool,
    account_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<(Vec<MediaObjectSummary>, i64), AppError> {
    if !(1..=MEDIA_MAX_PAGE_SIZE).contains(&limit)
        || offset.is_negative()
        || offset > MEDIA_MAX_PAGE_OFFSET
    {
        return Err(CommunicationError::MediaInvalid.into());
    }
    let rows = sqlx::query_as::<_, MediaObjectSummary>(
        "SELECT object.id, object.owner_account_id, object.resource_type, object.resource_id,\
            object.object_key, object.original_filename, object.media_type, object.size_bytes,\
            object.checksum, object.processing_status, object.created_at, object.updated_at,\
            CASE WHEN object.resource_id IS NOT NULL THEN object.resource_id::TEXT ELSE NULL END AS authorization_resource\
         FROM media_objects object WHERE object.owner_account_id = $1\
         ORDER BY object.created_at DESC, object.id DESC LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)::BIGINT FROM media_objects WHERE owner_account_id = $1",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((rows, total))
}

pub async fn claim_processing_job(
    pool: &PgPool,
    _worker_id: &str,
) -> Result<Option<Uuid>, AppError> {
    let processing = MediaProcessingJobStatus::Processing.as_db_value();
    let pending = MediaProcessingJobStatus::Pending.as_db_value();
    let failed = MediaProcessingJobStatus::Failed.as_db_value();
    let job: Option<Uuid> = sqlx::query_scalar(
        "UPDATE media_processing_jobs SET status = $1, attempt_count = attempt_count + 1,\
            claimed_until = now() + make_interval(secs => $4)\
         WHERE id = (\
             SELECT id FROM media_processing_jobs\
             WHERE status IN ($2, $3) AND available_at <= now()\
             ORDER BY created_at ASC FOR UPDATE SKIP LOCKED LIMIT 1\
         ) RETURNING id",
    )
    .bind(processing)
    .bind(pending)
    .bind(failed)
    .bind(MEDIA_PROCESSING_CLAIM_SECONDS)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(job)
}

pub async fn complete_processing_job(pool: &PgPool, job_id: Uuid) -> Result<(), AppError> {
    let completed = MediaProcessingJobStatus::Completed.as_db_value();
    let processing = MediaProcessingJobStatus::Processing.as_db_value();
    let ready = MediaProcessingStatus::Ready.as_db_value();
    let uploaded = MediaProcessingStatus::Uploaded.as_db_value();
    sqlx::query(
        "UPDATE media_processing_jobs SET status = $1, claimed_until = NULL\
         WHERE id = $2 AND status = $3",
    )
    .bind(completed)
    .bind(job_id)
    .bind(processing)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE media_objects SET processing_status = $1, updated_at = now()\
         WHERE id = (SELECT media_object_id FROM media_processing_jobs WHERE id = $2)\
           AND processing_status = $3",
    )
    .bind(ready)
    .bind(job_id)
    .bind(uploaded)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn fail_processing_job(
    pool: &PgPool,
    job_id: Uuid,
    error: &str,
    max_attempts: i32,
) -> Result<(), AppError> {
    let failed = MediaProcessingJobStatus::Failed.as_db_value();
    let pending = MediaProcessingJobStatus::Pending.as_db_value();
    let processing = MediaProcessingJobStatus::Processing.as_db_value();
    sqlx::query(
        "UPDATE media_processing_jobs SET status = CASE WHEN attempt_count >= $2 THEN $3 ELSE $4 END,\
            available_at = now() + make_interval(secs => $5), claimed_until = NULL, last_error = $6\
         WHERE id = $1 AND status = $7",
    )
    .bind(job_id)
    .bind(max_attempts)
    .bind(failed)
    .bind(pending)
    .bind(MEDIA_PROCESSING_CLAIM_SECONDS)
    .bind(error)
    .bind(processing)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn quarantine_media(pool: &PgPool, media_id: Uuid, reason: &str) -> Result<(), AppError> {
    let _ = reason;
    let quarantined = MediaProcessingStatus::Quarantined.as_db_value();
    let requested = MediaProcessingStatus::Requested.as_db_value();
    let uploading = MediaProcessingStatus::Uploading.as_db_value();
    let uploaded = MediaProcessingStatus::Uploaded.as_db_value();
    let processing = MediaProcessingStatus::Processing.as_db_value();
    let ready = MediaProcessingStatus::Ready.as_db_value();
    let result = sqlx::query(
        "UPDATE media_objects SET processing_status = $1, updated_at = now()\
         WHERE id = $2 AND processing_status IN ($3, $4, $5, $6, $7)\
         RETURNING id",
    )
    .bind(quarantined)
    .bind(media_id)
    .bind(requested)
    .bind(uploading)
    .bind(uploaded)
    .bind(processing)
    .bind(ready)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.is_some() {
        return Ok(());
    }
    Err(CommunicationError::MediaNotFound.into())
}

pub async fn release_media(pool: &PgPool, media_id: Uuid) -> Result<(), AppError> {
    let ready = MediaProcessingStatus::Ready.as_db_value();
    let quarantined = MediaProcessingStatus::Quarantined.as_db_value();
    let result = sqlx::query(
        "UPDATE media_objects SET processing_status = $1, updated_at = now()\
         WHERE id = $2 AND processing_status = $3 RETURNING id",
    )
    .bind(ready)
    .bind(media_id)
    .bind(quarantined)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.is_some() {
        return Ok(());
    }
    Err(CommunicationError::MediaNotFound.into())
}

pub async fn get_admin_media(
    pool: &PgPool,
    media_id: Uuid,
) -> Result<MediaObjectSummary, AppError> {
    sqlx::query_as::<_, MediaObjectSummary>(
        "SELECT object.id, object.owner_account_id, object.resource_type, object.resource_id,\
            object.object_key, object.original_filename, object.media_type, object.size_bytes,\
            object.checksum, object.processing_status, object.created_at, object.updated_at,\
            CASE WHEN object.resource_id IS NOT NULL THEN object.resource_id::TEXT ELSE NULL END AS authorization_resource\
         FROM media_objects object WHERE object.id = $1",
    )
    .bind(media_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::MediaNotFound.into())
}
