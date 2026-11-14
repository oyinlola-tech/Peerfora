use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;

use crate::{
    enums::authentication_rate_limit_operation::AuthenticationRateLimitOperation,
    utils::email_address::normalize_email_address,
};

pub async fn record_authentication_request(
    pool: &PgPool,
    operation: AuthenticationRateLimitOperation,
    email: &str,
    hash_key: &str,
) -> Result<bool, sqlx::Error> {
    let normalized_email = normalize_email_address(email);
    let mut mac = Hmac::<Sha256>::new_from_slice(hash_key.as_bytes())
        .map_err(|_| sqlx::Error::Protocol("invalid authentication rate-limit key".to_owned()))?;
    mac.update(normalized_email.as_bytes());
    let subject_hash = hex::encode(mac.finalize().into_bytes());
    let request_count = sqlx::query_scalar::<_, i32>(
        "WITH expired_buckets AS (
            DELETE FROM authentication_rate_limit_buckets
            WHERE expires_at <= now()
        )
        INSERT INTO authentication_rate_limit_buckets (
            operation, subject_hash, window_started_at, request_count, expires_at
        ) VALUES ($1, $2, now(), 1, now() + make_interval(secs => $3))
        ON CONFLICT (operation, subject_hash) DO UPDATE SET
            request_count = CASE
                WHEN authentication_rate_limit_buckets.window_started_at
                    <= now() - make_interval(secs => $3) THEN 1
                ELSE authentication_rate_limit_buckets.request_count + 1
            END,
            window_started_at = CASE
                WHEN authentication_rate_limit_buckets.window_started_at
                    <= now() - make_interval(secs => $3) THEN now()
                ELSE authentication_rate_limit_buckets.window_started_at
            END,
            expires_at = now() + make_interval(secs => $3)
        RETURNING request_count",
    )
    .bind(operation.database_key())
    .bind(subject_hash)
    .bind(operation.window_seconds())
    .fetch_one(pool)
    .await?;

    Ok(request_count <= operation.maximum_requests())
}
