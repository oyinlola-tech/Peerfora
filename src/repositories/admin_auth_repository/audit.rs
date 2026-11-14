use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::errors::app_error::AppError;

pub(crate) struct AdministrativeAuditEvent<'a> {
    pub actor_id: Uuid,
    pub action: &'a str,
    pub target_type: &'a str,
    pub target_id: Uuid,
    pub result: &'a str,
    pub reason: Option<&'a str>,
    pub correlation_id: Option<&'a str>,
}

pub(crate) async fn insert_administrative_audit_event(
    transaction: &mut Transaction<'_, Postgres>,
    event: AdministrativeAuditEvent<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id,
                result, reason, correlation_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(Uuid::new_v4())
    .bind(event.actor_id)
    .bind(event.action)
    .bind(event.target_type)
    .bind(event.target_id)
    .bind(event.result)
    .bind(event.reason)
    .bind(event.correlation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
