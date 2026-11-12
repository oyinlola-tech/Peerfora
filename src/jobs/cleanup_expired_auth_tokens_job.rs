use sqlx::PgPool;
use tracing::error;

use crate::repositories::account_repository::delete_expired_account_auth_tokens;

pub async fn cleanup_expired_auth_tokens(pool: &PgPool) {
    if delete_expired_account_auth_tokens(pool).await.is_err() {
        error!("Expired authentication data cleanup failed");
    }
}
