use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::errors::app_error::AppError;

pub async fn connect_database(database_url: &str) -> Result<PgPool, AppError> {
    if database_url.is_empty() {
        return Err(AppError::DatabaseConfigurationMissing);
    }
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|_| AppError::DatabaseUnavailable)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|_| AppError::DatabaseMigrationFailed)?;
    Ok(pool)
}
