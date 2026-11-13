use std::time::Duration;

use sqlx::PgPool;

use crate::errors::app_error::AppError;

pub async fn check_database_readiness(pool: &PgPool) -> Result<(), AppError> {
    tokio::time::timeout(
        Duration::from_secs(crate::constants::health::DATABASE_READINESS_TIMEOUT_SECONDS),
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool),
    )
    .await
    .map_err(|_| AppError::DatabaseUnavailable)?
    .map(|_| ())
    .map_err(|_| AppError::DatabaseUnavailable)
}
