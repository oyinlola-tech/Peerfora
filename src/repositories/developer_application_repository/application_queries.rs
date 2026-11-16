use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::developer::DeveloperApplicationStatus,
    errors::app_error::AppError,
    models::developer::{AdministrativeDeveloperApplicationRecord, DeveloperApplicationRecord},
};

pub async fn list_applications_for_owner(
    pool: &PgPool,
    owner_account_id: Uuid,
) -> Result<Vec<DeveloperApplicationRecord>, AppError> {
    sqlx::query_as::<_, DeveloperApplicationRecord>(
        "SELECT application.id, application.name, application.description,
            application.application_type, application.application_status,
            COALESCE((SELECT jsonb_agg(jsonb_build_object(
                'scopeKey', requested.scope_key, 'scopeStatus', requested.scope_status)
                ORDER BY requested.scope_key)
             FROM developer_application_requested_scopes requested
             WHERE requested.application_id = application.id), '[]'::jsonb) AS scopes,
            application.submitted_at, application.reviewed_at, application.review_reason,
            application.created_at, application.updated_at
         FROM developer_applications application
         WHERE application.owner_account_id = $1
         ORDER BY application.created_at DESC, application.id",
    )
    .bind(owner_account_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_applications_for_review(
    pool: &PgPool,
    status: Option<DeveloperApplicationStatus>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AdministrativeDeveloperApplicationRecord>, i64), AppError> {
    let status_key = status.map(DeveloperApplicationStatus::as_database_value);
    let total_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM developer_applications application
         WHERE ($1::text IS NULL OR application.application_status = $1)",
    )
    .bind(status_key)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let rows = sqlx::query_as::<_, AdministrativeDeveloperApplicationRecord>(
        "SELECT application.id, application.owner_account_id,
            account.username AS owner_username, application.name, application.description,
            application.application_type, application.application_status,
            COALESCE((SELECT jsonb_agg(jsonb_build_object(
                'scopeKey', requested.scope_key, 'scopeStatus', requested.scope_status)
                ORDER BY requested.scope_key)
             FROM developer_application_requested_scopes requested
             WHERE requested.application_id = application.id), '[]'::jsonb) AS scopes,
            application.submitted_at, application.created_at
         FROM developer_applications application
         JOIN accounts account ON account.id = application.owner_account_id
         WHERE ($1::text IS NULL OR application.application_status = $1)
         ORDER BY application.created_at DESC, application.id
         LIMIT $2 OFFSET $3",
    )
    .bind(status_key)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((rows, total_count))
}
