use sqlx::PgPool;

use crate::{
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    errors::app_error::AppError,
    repositories::authentication_rate_limit_repository::record_authentication_request,
};

pub async fn enforce_authentication_rate_limit(
    pool: &PgPool,
    operation: AuthenticationRateLimitOperation,
    email: &str,
    hash_key: &str,
) -> Result<(), AppError> {
    let allowed = record_authentication_request(pool, operation, email, hash_key)
        .await
        .map_err(|error| {
            tracing::error!(%error, "authentication rate limit storage failed");
            AppError::Internal
        })?;

    if allowed {
        Ok(())
    } else {
        Err(AppError::AuthenticationRateLimited)
    }
}
