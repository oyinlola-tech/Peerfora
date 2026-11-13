use sqlx::PgPool;

use crate::errors::app_error::AppError;

pub async fn check_database_readiness(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
        .map_err(|_| AppError::DatabaseUnavailable)
}
