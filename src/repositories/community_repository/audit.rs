use sqlx::{Error, Postgres, Transaction};
use uuid::Uuid;

use crate::{enums::community_error::CommunityError, errors::app_error::AppError};

pub(super) async fn record_community_change(
    transaction: &mut Transaction<'_, Postgres>,
    community_id: Uuid,
    actor_account_id: Uuid,
    target_account_id: Option<Uuid>,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO community_audit_events (
            community_id, actor_account_id, event_type, target_account_id, payload
        ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(community_id)
    .bind(actor_account_id)
    .bind(event_type)
    .bind(target_account_id)
    .bind(&payload)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    sqlx::query(
        "INSERT INTO outbox_events (event_type, aggregate_type, aggregate_id, payload)
         VALUES ($1, 'community', $2, $3)",
    )
    .bind(event_type)
    .bind(community_id)
    .bind(payload)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok(())
}

pub(super) fn map_community_write_error(error: Error) -> AppError {
    if let Some(database_error) = error.as_database_error()
        && database_error.code().is_some_and(|code| code == "23505")
    {
        return CommunityError::CommunitySlugAlreadyUsed.into();
    }
    AppError::Internal
}
