use sqlx::PgPool;

use crate::errors::app_error::AppError;

pub async fn is_active_country_code(pool: &PgPool, country_code: &str) -> Result<bool, AppError> {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM countries
            WHERE country_code = $1 AND is_active = TRUE
        )",
    )
    .bind(country_code)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}
