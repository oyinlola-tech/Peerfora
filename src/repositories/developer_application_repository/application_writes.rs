use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::developer::{DeveloperApplicationScope, DeveloperError},
    errors::app_error::AppError,
    models::developer::DeveloperApplicationRecord,
};

pub async fn create_developer_application(
    pool: &PgPool,
    owner_account_id: Uuid,
    name: &str,
    description: &str,
    application_type: &str,
    requested_scopes: &[DeveloperApplicationScope],
) -> Result<DeveloperApplicationRecord, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let profile_is_active: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM developer_profiles profile
         JOIN accounts account ON account.id = profile.account_id
         WHERE profile.account_id = $1 AND profile.profile_status = 'active'
             AND account.account_status = 'active' AND account.email_verified_at IS NOT NULL)",
    )
    .bind(owner_account_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !profile_is_active {
        return Err(DeveloperError::ProfileUnavailable.into());
    }

    let scope_keys = requested_scopes
        .iter()
        .map(|scope| scope.as_database_value().to_owned())
        .collect::<Vec<_>>();
    let active_scope_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM developer_application_scopes
         WHERE scope_key = ANY($1::text[]) AND is_active = TRUE",
    )
    .bind(&scope_keys)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_scope_count != scope_keys.len() as i64 {
        return Err(DeveloperError::RequestedScopeUnavailable.into());
    }

    let application_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO developer_applications
            (id, owner_account_id, name, description, application_type)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(application_id)
    .bind(owner_account_id)
    .bind(name)
    .bind(description)
    .bind(application_type)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        if error
            .as_database_error()
            .and_then(|database| database.constraint())
            == Some("developer_applications_owner_name_lower_unique")
        {
            DeveloperError::ApplicationAlreadyExists.into()
        } else {
            AppError::Internal
        }
    })?;

    sqlx::query(
        "INSERT INTO developer_application_requested_scopes (application_id, scope_key)
         SELECT $1, scope_key FROM unnest($2::text[]) AS scope_key",
    )
    .bind(application_id)
    .bind(scope_keys)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    find_created_application(pool, application_id, owner_account_id).await
}

pub async fn submit_developer_application(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE developer_applications application
         SET application_status = 'submitted', submitted_at = now()
         WHERE application.id = $1 AND application.owner_account_id = $2
             AND application.application_status = 'draft'
             AND EXISTS (SELECT 1 FROM developer_application_requested_scopes scope
                         WHERE scope.application_id = application.id)",
    )
    .bind(application_id)
    .bind(owner_account_id)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() == 1 {
        return Ok(());
    }
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM developer_applications
         WHERE id = $1 AND owner_account_id = $2)",
    )
    .bind(application_id)
    .bind(owner_account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if exists {
        Err(DeveloperError::ApplicationCannotBeSubmitted.into())
    } else {
        Err(DeveloperError::ApplicationNotFound.into())
    }
}

async fn find_created_application(
    pool: &PgPool,
    application_id: Uuid,
    owner_account_id: Uuid,
) -> Result<DeveloperApplicationRecord, AppError> {
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
         WHERE application.id = $1 AND application.owner_account_id = $2",
    )
    .bind(application_id)
    .bind(owner_account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}
