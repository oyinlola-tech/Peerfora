use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::developers::{
        DEVELOPER_CREDENTIAL_LAST_USED_UPDATE_MINUTES, DEVELOPER_CREDENTIAL_MAX_REQUESTS_PER_MINUTE,
    },
    enums::developer::{DeveloperCredentialEnvironment, DeveloperCredentialValidation},
    errors::app_error::AppError,
};

pub async fn validate_developer_credential_for_scope(
    pool: &PgPool,
    token_hash: &str,
    expected_environment: DeveloperCredentialEnvironment,
    required_scope: &str,
) -> Result<DeveloperCredentialValidation, AppError> {
    let credential: Option<(Uuid, Uuid, bool)> = sqlx::query_as(
        "SELECT credential.id, credential.application_id,
            EXISTS (
                SELECT 1 FROM developer_credential_scopes credential_scope
                JOIN developer_application_requested_scopes application_scope
                    ON application_scope.application_id = credential_scope.application_id
                    AND application_scope.scope_key = credential_scope.scope_key
                WHERE credential_scope.credential_id = credential.id
                    AND credential_scope.scope_key = $3
                    AND application_scope.scope_status = 'approved'
            ) AS has_scope
         FROM developer_application_credentials credential
         JOIN developer_applications application
             ON application.id = credential.application_id
         JOIN developer_profiles profile
             ON profile.account_id = application.owner_account_id
         JOIN accounts account ON account.id = application.owner_account_id
         WHERE credential.token_hash = $1
             AND credential.environment = $2
             AND credential.revoked_at IS NULL
             AND credential.expires_at > now()
             AND application.application_status = 'approved'
             AND profile.profile_status = 'active'
             AND account.account_status = 'active'
             AND account.email_verified_at IS NOT NULL",
    )
    .bind(token_hash)
    .bind(expected_environment.as_database_value())
    .bind(required_scope)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some((credential_id, application_id, has_scope)) = credential else {
        return Ok(DeveloperCredentialValidation::Invalid);
    };
    if !has_scope {
        return Ok(DeveloperCredentialValidation::ScopeDenied);
    }

    let request_count = increment_credential_minute_count(pool, credential_id).await?;
    if request_count > DEVELOPER_CREDENTIAL_MAX_REQUESTS_PER_MINUTE {
        return Ok(DeveloperCredentialValidation::RateLimited);
    }
    update_credential_last_used(pool, credential_id).await?;
    Ok(DeveloperCredentialValidation::Authorized(
        credential_id,
        application_id,
    ))
}

async fn increment_credential_minute_count(
    pool: &PgPool,
    credential_id: Uuid,
) -> Result<i32, AppError> {
    sqlx::query_scalar(
        "INSERT INTO developer_credential_rate_limits
            (credential_id, window_started_at, request_count)
         VALUES ($1, now(), 1)
         ON CONFLICT (credential_id) DO UPDATE
         SET window_started_at = CASE
                 WHEN developer_credential_rate_limits.window_started_at < now() - interval '1 minute'
                 THEN now()
                 ELSE developer_credential_rate_limits.window_started_at END,
             request_count = CASE
                 WHEN developer_credential_rate_limits.window_started_at < now() - interval '1 minute'
                 THEN 1
                 ELSE LEAST(developer_credential_rate_limits.request_count + 1, $2) END
         RETURNING request_count",
    )
    .bind(credential_id)
    .bind(DEVELOPER_CREDENTIAL_MAX_REQUESTS_PER_MINUTE + 1)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

async fn update_credential_last_used(pool: &PgPool, credential_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE developer_application_credentials
         SET last_used_at = now()
         WHERE id = $1 AND (last_used_at IS NULL OR last_used_at <
             now() - make_interval(mins => $2::integer))",
    )
    .bind(credential_id)
    .bind(DEVELOPER_CREDENTIAL_LAST_USED_UPDATE_MINUTES as i32)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
