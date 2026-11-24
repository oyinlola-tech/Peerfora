use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{enums::communication::OutboxProcessingStatus, errors::app_error::AppError};

#[derive(sqlx::FromRow)]
pub struct OutboxEventSummary {
    pub id: Uuid,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub processing_status: String,
    pub attempt_count: i32,
    pub occurred_at: DateTime<Utc>,
}

pub async fn publish_outbox_event(
    pool: &PgPool,
    event_type: &str,
    aggregate_type: &str,
    aggregate_id: Uuid,
    payload: &Value,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO outbox_events (event_type, aggregate_type, aggregate_id, payload)\
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(event_type)
    .bind(aggregate_type)
    .bind(aggregate_id)
    .bind(payload)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn claim_outbox_event(
    pool: &PgPool,
    worker_id: &str,
    claim_token: Uuid,
    claim_seconds: i64,
) -> Result<Option<(Uuid, String, Value)>, AppError> {
    let processing = OutboxProcessingStatus::Processing.as_db_value();
    let pending = OutboxProcessingStatus::Pending.as_db_value();
    let failed = OutboxProcessingStatus::Failed.as_db_value();
    let claimed = sqlx::query_as::<_, (Uuid, String, Value)>(
        "UPDATE outbox_events SET processing_status = $1, attempt_count = attempt_count + 1,\
            claimed_by = $2, claim_token = $3, claimed_until = now() + make_interval(secs => $4),\
            last_error = NULL \
         WHERE id = (\
             SELECT id FROM outbox_events \
             WHERE processing_status IN ($5, $6) AND available_at <= now()\
             ORDER BY occurred_at ASC, id ASC FOR UPDATE SKIP LOCKED LIMIT 1 \
         ) RETURNING id, event_type, payload",
    )
    .bind(processing)
    .bind(worker_id)
    .bind(claim_token)
    .bind(claim_seconds)
    .bind(pending)
    .bind(failed)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(claimed)
}

pub async fn mark_outbox_processed(
    pool: &PgPool,
    event_id: Uuid,
    worker_id: &str,
) -> Result<bool, AppError> {
    let processed = OutboxProcessingStatus::Processed.as_db_value();
    let processing = OutboxProcessingStatus::Processing.as_db_value();
    let result = sqlx::query(
        "UPDATE outbox_events SET processing_status = $1, processed_at = now(),\
            claimed_by = NULL, claim_token = NULL, claimed_until = NULL \
         WHERE id = $2 AND processing_status = $3 AND claimed_by = $4 \
         RETURNING id",
    )
    .bind(processed)
    .bind(event_id)
    .bind(processing)
    .bind(worker_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(result.is_some())
}

pub async fn mark_outbox_failed(
    pool: &PgPool,
    event_id: Uuid,
    worker_id: &str,
    error: &str,
    available_in_seconds: i64,
    max_attempts: i32,
) -> Result<bool, AppError> {
    let failed = OutboxProcessingStatus::Failed.as_db_value();
    let processing = OutboxProcessingStatus::Processing.as_db_value();
    let result = sqlx::query(
        "UPDATE outbox_events SET processing_status = $1, last_error = $2,\
            available_at = now() + make_interval(secs => $3),\
            claimed_by = NULL, claim_token = NULL, claimed_until = NULL \
         WHERE id = $4 AND processing_status = $5 AND claimed_by = $6 \
         RETURNING id",
    )
    .bind(failed)
    .bind(error)
    .bind(available_in_seconds)
    .bind(event_id)
    .bind(processing)
    .bind(worker_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let attempts =
        sqlx::query_scalar::<_, i32>("SELECT attempt_count FROM outbox_events WHERE id = $1")
            .bind(event_id)
            .fetch_one(pool)
            .await
            .map_err(|_| AppError::Internal)?;
    if result.is_some() && attempts >= max_attempts {
        sqlx::query("DELETE FROM outbox_events WHERE id = $1 AND attempt_count >= $2")
            .bind(event_id)
            .bind(max_attempts)
            .execute(pool)
            .await
            .map_err(|_| AppError::Internal)?;
    }
    Ok(result.is_some())
}

pub async fn get_outbox_event(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<OutboxEventSummary, AppError> {
    sqlx::query_as::<_, OutboxEventSummary>(
        "SELECT id, event_type, aggregate_type, aggregate_id, processing_status,\
            attempt_count, occurred_at FROM outbox_events WHERE id = $1",
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AppError::Internal)
}

pub async fn publish_email_delivery(
    pool: &PgPool,
    recipient: &str,
    encrypted_payload: &str,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO email_delivery_outbox (recipient, encrypted_payload)\
         VALUES ($1, $2) RETURNING id",
    )
    .bind(recipient)
    .bind(encrypted_payload)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn claim_email_delivery(
    pool: &PgPool,
    _worker_id: &str,
    claim_seconds: i64,
) -> Result<Option<(Uuid, String, String)>, AppError> {
    let processing = OutboxProcessingStatus::Processing.as_db_value();
    let pending = OutboxProcessingStatus::Pending.as_db_value();
    let failed = OutboxProcessingStatus::Failed.as_db_value();
    let claimed = sqlx::query_as::<_, (Uuid, String, String)>(
        "UPDATE email_delivery_outbox SET processing_status = $1, attempt_count = attempt_count + 1,\
            claimed_until = now() + make_interval(secs => $2)\
         WHERE id = (\
             SELECT id FROM email_delivery_outbox \
             WHERE processing_status IN ($3, $4) AND available_at <= now()\
             ORDER BY created_at ASC FOR UPDATE SKIP LOCKED LIMIT 1 \
         ) RETURNING id, recipient, encrypted_payload",
    )
    .bind(processing)
    .bind(claim_seconds)
    .bind(pending)
    .bind(failed)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(claimed)
}

pub async fn mark_email_delivery_processed(
    pool: &PgPool,
    delivery_id: Uuid,
    _worker_id: &str,
) -> Result<bool, AppError> {
    let processed = OutboxProcessingStatus::Processed.as_db_value();
    let processing = OutboxProcessingStatus::Processing.as_db_value();
    let result = sqlx::query_scalar::<_, bool>(
        "UPDATE email_delivery_outbox SET processing_status = $1, processed_at = now(),\
            claimed_until = NULL \
         WHERE id = $2 AND processing_status = $3 \
             AND claimed_until > now() RETURNING TRUE",
    )
    .bind(processed)
    .bind(delivery_id)
    .bind(processing)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(result.unwrap_or(false))
}

pub async fn mark_email_delivery_failed(
    pool: &PgPool,
    delivery_id: Uuid,
    error: &str,
    available_in_seconds: i64,
    max_attempts: i32,
) -> Result<(), AppError> {
    let failed = OutboxProcessingStatus::Failed.as_db_value();
    let processing = OutboxProcessingStatus::Processing.as_db_value();
    sqlx::query(
        "UPDATE email_delivery_outbox SET processing_status = $1, last_error = $2,\
            available_at = now() + make_interval(secs => $3), claimed_until = NULL \
         WHERE id = $4 AND processing_status = $5",
    )
    .bind(failed)
    .bind(error)
    .bind(available_in_seconds)
    .bind(delivery_id)
    .bind(processing)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let attempts = sqlx::query_scalar::<_, i32>(
        "SELECT attempt_count FROM email_delivery_outbox WHERE id = $1",
    )
    .bind(delivery_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if attempts >= max_attempts {
        sqlx::query("DELETE FROM email_delivery_outbox WHERE id = $1 AND attempt_count >= $2")
            .bind(delivery_id)
            .bind(max_attempts)
            .execute(pool)
            .await
            .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}
