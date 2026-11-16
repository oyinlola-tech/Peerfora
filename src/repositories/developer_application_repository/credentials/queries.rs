use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::developer::DeveloperError, errors::app_error::AppError,
    models::developer::DeveloperCredentialRecord,
};

pub async fn list_developer_credentials_for_owner(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
) -> Result<Vec<DeveloperCredentialRecord>, AppError> {
    let application_is_owned: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM developer_applications
         WHERE id = $1 AND owner_account_id = $2)",
    )
    .bind(application_id)
    .bind(owner_account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if !application_is_owned {
        return Err(DeveloperError::ApplicationNotFound.into());
    }
    sqlx::query_as::<_, DeveloperCredentialRecord>(
        "SELECT credential.id, credential.application_id, credential.name,
            credential.environment, credential.created_at, credential.expires_at,
            credential.revoked_at, credential.last_used_at,
            ARRAY(SELECT scope.scope_key FROM developer_credential_scopes scope
                  WHERE scope.credential_id = credential.id ORDER BY scope.scope_key) AS scopes
         FROM developer_application_credentials credential
         JOIN developer_applications application
             ON application.id = credential.application_id
         WHERE credential.application_id = $1 AND application.owner_account_id = $2
         ORDER BY credential.created_at DESC, credential.id",
    )
    .bind(application_id)
    .bind(owner_account_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
