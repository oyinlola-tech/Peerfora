use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::phase_four::{DEVICE_PUSH_MAX_TOKEN_LENGTH, DEVICE_PUSH_MIN_TOKEN_LENGTH},
    enums::communication::DevicePushPlatform,
    enums::phase_four::CommunicationError,
    errors::app_error::AppError,
    models::communication::DevicePushTokenSummary,
};

pub fn validate_push_token(platform: &str, token: &str) -> Result<(), CommunicationError> {
    if !DevicePushPlatform::is_supported(platform)
        || token.trim().is_empty()
        || token.len() < DEVICE_PUSH_MIN_TOKEN_LENGTH
        || token.len() > DEVICE_PUSH_MAX_TOKEN_LENGTH
    {
        return Err(CommunicationError::DevicePushInvalid);
    }
    Ok(())
}

pub async fn register_device_token(
    pool: &PgPool,
    account_id: Uuid,
    platform: &str,
    token: &str,
) -> Result<Uuid, AppError> {
    validate_push_token(platform, token)?;
    let token_trimmed = token.trim();
    let existing: Option<(Uuid, bool)> = sqlx::query_as(
        "SELECT id, active FROM device_push_tokens WHERE token = $1 AND account_id = $2",
    )
    .bind(token_trimmed)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    match existing {
        Some((id, active)) => {
            sqlx::query("UPDATE device_push_tokens SET active = TRUE, updated_at = now() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await
                .map_err(|_| AppError::Internal)?;
            Ok(id)
        }
        None => sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO device_push_tokens (account_id, platform, token, active)\n         VALUES ($1, $2, $3, TRUE) RETURNING id",
        )
        .bind(account_id)
        .bind(platform)
        .bind(token_trimmed)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Internal),
    }
}

pub async fn update_device_token(
    pool: &PgPool,
    account_id: Uuid,
    token_id: Uuid,
    platform: &str,
    active: bool,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE device_push_tokens SET platform = $3, active = $4, updated_at = now()\n         WHERE id = $1 AND account_id = $2 RETURNING id",
    )
    .bind(token_id)
    .bind(account_id)
    .bind(platform)
    .bind(active)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.is_some() {
        return Ok(());
    }
    Err(CommunicationError::DevicePushNotFound.into())
}

pub async fn list_device_tokens(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<DevicePushTokenSummary>, AppError> {
    sqlx::query_as::<_, DevicePushTokenSummary>(
        "SELECT id, account_id, platform, active, created_at, updated_at\
         FROM device_push_tokens WHERE account_id = $1\
         ORDER BY created_at DESC, id DESC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn get_device_token(
    pool: &PgPool,
    account_id: Uuid,
    token_id: Uuid,
) -> Result<DevicePushTokenSummary, AppError> {
    sqlx::query_as::<_, DevicePushTokenSummary>(
        "SELECT id, account_id, platform, active, created_at, updated_at\
         FROM device_push_tokens WHERE id = $1 AND account_id = $2",
    )
    .bind(token_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::DevicePushNotFound.into())
}

pub async fn upsert_realtime_cursor(
    pool: &PgPool,
    account_id: Uuid,
    last_event_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO realtime_event_cursors (account_id, last_event_id, updated_at)\n         VALUES ($1, $2, now())\n         ON CONFLICT (account_id) DO UPDATE SET last_event_id = EXCLUDED.last_event_id,\
            updated_at = now()",
    )
    .bind(account_id)
    .bind(last_event_id)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn get_realtime_cursor(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT last_event_id FROM realtime_event_cursors WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn touch_device_token(pool: &PgPool, token_id: Uuid) -> Result<DateTime<Utc>, AppError> {
    sqlx::query_scalar::<_, DateTime<Utc>>(
        "UPDATE device_push_tokens SET updated_at = now() WHERE id = $1 RETURNING updated_at",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::DevicePushNotFound.into())
}
