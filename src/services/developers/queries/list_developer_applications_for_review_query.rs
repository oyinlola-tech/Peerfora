use sqlx::PgPool;

use crate::{
    enums::developer::DeveloperApplicationStatus, errors::app_error::AppError,
    models::developer::AdministrativeDeveloperApplicationRecord,
    repositories::developer_application_repository,
};

pub async fn list_developer_applications_for_review(
    pool: &PgPool,
    status: Option<DeveloperApplicationStatus>,
    limit: u32,
    offset: u32,
) -> Result<(Vec<AdministrativeDeveloperApplicationRecord>, i64), AppError> {
    developer_application_repository::list_applications_for_review(
        pool,
        status,
        i64::from(limit),
        i64::from(offset),
    )
    .await
}
