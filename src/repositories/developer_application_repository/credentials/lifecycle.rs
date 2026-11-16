use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::developers::DEVELOPER_CREDENTIAL_MAX_ACTIVE_PER_APPLICATION,
    enums::developer::{DeveloperCredentialEnvironment, DeveloperError},
    errors::app_error::AppError,
    models::developer::DeveloperCredentialRecord,
};

pub struct IssueDeveloperCredential {
    pub owner_account_id: Uuid,
    pub application_id: Uuid,
    pub credential_id: Uuid,
    pub token_hash: String,
    pub name: String,
    pub environment: DeveloperCredentialEnvironment,
    pub expires_in_days: u16,
}

pub struct RotateDeveloperCredential {
    pub owner_account_id: Uuid,
    pub application_id: Uuid,
    pub previous_credential_id: Uuid,
    pub new_credential_id: Uuid,
    pub new_token_hash: String,
    pub expires_in_days: u16,
    pub environment: DeveloperCredentialEnvironment,
}

struct CredentialInsert<'a> {
    application_id: Uuid,
    credential_id: Uuid,
    token_hash: &'a str,
    name: &'a str,
    environment: DeveloperCredentialEnvironment,
    expires_in_days: u16,
    scopes: &'a [String],
}

pub async fn issue_developer_credential(
    pool: &PgPool,
    request: IssueDeveloperCredential,
) -> Result<DeveloperCredentialRecord, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_approved_application(
        &mut transaction,
        request.owner_account_id,
        request.application_id,
    )
    .await?;
    ensure_credential_capacity(&mut transaction, request.application_id).await?;
    let scopes = find_approved_scopes(&mut transaction, request.application_id).await?;
    if scopes.is_empty() {
        return Err(DeveloperError::NoApprovedScopes.into());
    }
    insert_credential(
        &mut transaction,
        CredentialInsert {
            application_id: request.application_id,
            credential_id: request.credential_id,
            token_hash: &request.token_hash,
            name: &request.name,
            environment: request.environment,
            expires_in_days: request.expires_in_days,
            scopes: &scopes,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    find_credential(pool, request.credential_id).await
}

pub async fn revoke_developer_credential(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
    credential_id: Uuid,
) -> Result<(), AppError> {
    let updated = sqlx::query(
        "UPDATE developer_application_credentials credential
         SET revoked_at = COALESCE(credential.revoked_at, now())
         FROM developer_applications application
         WHERE credential.id = $1 AND credential.application_id = $2
             AND application.id = credential.application_id
             AND application.owner_account_id = $3",
    )
    .bind(credential_id)
    .bind(application_id)
    .bind(owner_account_id)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if updated.rows_affected() == 1 {
        Ok(())
    } else {
        Err(DeveloperError::CredentialNotFound.into())
    }
}

pub async fn rotate_developer_credential(
    pool: &PgPool,
    request: RotateDeveloperCredential,
) -> Result<DeveloperCredentialRecord, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_approved_application(
        &mut transaction,
        request.owner_account_id,
        request.application_id,
    )
    .await?;
    let (name, previous_environment, previous_scopes) = find_active_credential_for_rotation(
        &mut transaction,
        request.application_id,
        request.previous_credential_id,
    )
    .await?;
    if previous_environment != request.environment.as_database_value() {
        return Err(DeveloperError::CredentialEnvironmentMismatch.into());
    }
    let approved_scopes = find_approved_scopes(&mut transaction, request.application_id).await?;
    let retained_scopes = previous_scopes
        .into_iter()
        .filter(|scope| approved_scopes.contains(scope))
        .collect::<Vec<_>>();
    if retained_scopes.is_empty() {
        return Err(DeveloperError::NoApprovedScopes.into());
    }

    sqlx::query(
        "UPDATE developer_application_credentials
         SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(request.previous_credential_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    ensure_credential_capacity(&mut transaction, request.application_id).await?;
    insert_credential(
        &mut transaction,
        CredentialInsert {
            application_id: request.application_id,
            credential_id: request.new_credential_id,
            token_hash: &request.new_token_hash,
            name: &name,
            environment: request.environment,
            expires_in_days: request.expires_in_days,
            scopes: &retained_scopes,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    find_credential(pool, request.new_credential_id).await
}

async fn lock_approved_application(
    transaction: &mut Transaction<'_, Postgres>,
    owner_account_id: Uuid,
    application_id: Uuid,
) -> Result<(), AppError> {
    let status: Option<String> = sqlx::query_scalar(
        "SELECT application.application_status
         FROM developer_applications application
         JOIN developer_profiles profile ON profile.account_id = application.owner_account_id
         JOIN accounts account ON account.id = application.owner_account_id
         WHERE application.id = $1 AND application.owner_account_id = $2
             AND profile.profile_status = 'active'
             AND account.account_status = 'active' AND account.email_verified_at IS NOT NULL
         FOR UPDATE OF application",
    )
    .bind(application_id)
    .bind(owner_account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    match status.as_deref() {
        None => Err(DeveloperError::ApplicationNotFound.into()),
        Some("approved") => Ok(()),
        Some("suspended") => Err(DeveloperError::ApplicationSuspended.into()),
        Some(_) => Err(DeveloperError::ApplicationNotApproved.into()),
    }
}

async fn ensure_credential_capacity(
    transaction: &mut Transaction<'_, Postgres>,
    application_id: Uuid,
) -> Result<(), AppError> {
    let active_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM developer_application_credentials
         WHERE application_id = $1 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(application_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_count >= DEVELOPER_CREDENTIAL_MAX_ACTIVE_PER_APPLICATION {
        return Err(DeveloperError::CredentialLimitReached.into());
    }
    Ok(())
}

async fn find_approved_scopes(
    transaction: &mut Transaction<'_, Postgres>,
    application_id: Uuid,
) -> Result<Vec<String>, AppError> {
    sqlx::query_scalar(
        "SELECT scope_key FROM developer_application_requested_scopes
         WHERE application_id = $1 AND scope_status = 'approved'
         ORDER BY scope_key",
    )
    .bind(application_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}

async fn insert_credential(
    transaction: &mut Transaction<'_, Postgres>,
    credential: CredentialInsert<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO developer_application_credentials
            (id, application_id, token_hash, name, environment, expires_at)
         VALUES ($1, $2, $3, $4, $5,
                 now() + make_interval(days => $6::integer))",
    )
    .bind(credential.credential_id)
    .bind(credential.application_id)
    .bind(credential.token_hash)
    .bind(credential.name)
    .bind(credential.environment.as_database_value())
    .bind(i32::from(credential.expires_in_days))
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    sqlx::query(
        "INSERT INTO developer_credential_scopes (credential_id, application_id, scope_key)
         SELECT $1, $2, requested.scope_key
         FROM developer_application_requested_scopes requested
         WHERE requested.application_id = $2 AND requested.scope_status = 'approved'
             AND requested.scope_key = ANY($3::text[])",
    )
    .bind(credential.credential_id)
    .bind(credential.application_id)
    .bind(credential.scopes)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn find_active_credential_for_rotation(
    transaction: &mut Transaction<'_, Postgres>,
    application_id: Uuid,
    credential_id: Uuid,
) -> Result<(String, String, Vec<String>), AppError> {
    let record: Option<(String, String, Vec<String>)> = sqlx::query_as(
        "SELECT credential.name, credential.environment,
            ARRAY(SELECT scope.scope_key FROM developer_credential_scopes scope
                  WHERE scope.credential_id = credential.id ORDER BY scope.scope_key)
         FROM developer_application_credentials credential
         WHERE credential.id = $1 AND credential.application_id = $2
             AND credential.revoked_at IS NULL AND credential.expires_at > now()
         FOR UPDATE",
    )
    .bind(credential_id)
    .bind(application_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    record.ok_or(DeveloperError::CredentialNotActive.into())
}

async fn find_credential(
    pool: &PgPool,
    credential_id: Uuid,
) -> Result<DeveloperCredentialRecord, AppError> {
    sqlx::query_as::<_, DeveloperCredentialRecord>(
        "SELECT credential.id, credential.application_id, credential.name,
            credential.environment, credential.created_at, credential.expires_at,
            credential.revoked_at, credential.last_used_at,
            ARRAY(SELECT scope.scope_key FROM developer_credential_scopes scope
                  WHERE scope.credential_id = credential.id ORDER BY scope.scope_key) AS scopes
         FROM developer_application_credentials credential WHERE credential.id = $1",
    )
    .bind(credential_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}
