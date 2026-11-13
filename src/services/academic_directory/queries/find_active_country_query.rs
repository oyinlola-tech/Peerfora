use sqlx::PgPool;

use crate::{errors::app_error::AppError, repositories::country_directory_repository};

pub async fn is_country_code_supported(
    pool: &PgPool,
    country_code: &str,
) -> Result<bool, AppError> {
    country_directory_repository::is_active_country_code(pool, country_code).await
}
