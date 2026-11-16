use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::developer_application_repository};

pub async fn submit_developer_application(
    pool: &PgPool,
    owner_account_id: Uuid,
    application_id: Uuid,
) -> Result<(), AppError> {
    developer_application_repository::submit_developer_application(
        pool,
        owner_account_id,
        application_id,
    )
    .await
}
