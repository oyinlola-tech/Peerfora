use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::developer::DeveloperApplicationRecord,
    repositories::developer_application_repository,
};

pub async fn list_developer_applications(
    pool: &PgPool,
    owner_account_id: Uuid,
) -> Result<Vec<DeveloperApplicationRecord>, AppError> {
    developer_application_repository::list_applications_for_owner(pool, owner_account_id).await
}
