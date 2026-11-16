use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::developer::DeveloperCredentialRecord,
    repositories::developer_application_repository::credentials,
};

pub async fn list_developer_credentials(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
) -> Result<Vec<DeveloperCredentialRecord>, AppError> {
    credentials::list_developer_credentials_for_owner(pool, owner_account_id, application_id).await
}
