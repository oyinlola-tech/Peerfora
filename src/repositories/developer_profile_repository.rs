use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, models::developer::DeveloperProfileRecord};

pub async fn create_developer_profile(
    pool: &PgPool,
    account_id: Uuid,
    display_name: &str,
    description: &str,
) -> Result<DeveloperProfileRecord, AppError> {
    sqlx::query_as::<_, DeveloperProfileRecord>(
        "INSERT INTO developer_profiles (account_id, display_name, description)
         SELECT account.id, $2, $3 FROM accounts account
         WHERE account.id = $1 AND account.account_status = 'active'
             AND account.email_verified_at IS NOT NULL
         RETURNING account_id, display_name, description, profile_status, created_at",
    )
    .bind(account_id)
    .bind(display_name)
    .bind(description)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        if error
            .as_database_error()
            .and_then(|database| database.constraint())
            == Some("developer_profiles_pkey")
        {
            crate::enums::developer::DeveloperError::ProfileAlreadyExists.into()
        } else {
            AppError::Internal
        }
    })?
    .ok_or(crate::enums::developer::DeveloperError::ProfileUnavailable.into())
}
