use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::MEDIA_PROCESSING_CLAIM_SECONDS,
    enums::communication::{MediaProcessingJobStatus, MediaProcessingStatus},
    errors::app_error::AppError,
};

/// Claim the next runnable media-processing job, marking it processing.
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

/// Mark a claimed media-processing job complete and promote its object to ready.
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

/// Mark a claimed media-processing job failed, retrying until attempts run out.
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
