use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_SENSITIVE_OPERATION_RECENT_AUTH_MINUTES, ADMIN_SESSION_IDLE_TIMEOUT_MINUTES,
        ADMIN_SESSION_LIFETIME_HOURS, ADMINISTRATOR_RECOVERY_CODES_REGENERATED_AUDIT_ACTION,
    },
    enums::admin_auth::AdministratorMfaChallengePurpose,
    errors::app_error::AppError,
    models::admin_identity::AdministratorMfaSecretRecord,
};

use super::{
    audit::{AdministrativeAuditEvent, insert_administrative_audit_event},
    identity::record_failed_administrator_mfa_attempt,
};

pub struct AdministratorMfaSessionCreation<'a> {
    pub challenge_hash: &'a str,
    pub totp_step: u64,
    pub enrolled_secret: Option<&'a str>,
    pub recovery_code_hashes: &'a [String],
    pub session_id: Uuid,
    pub session_token_hash: &'a str,
    pub correlation_id: Option<&'a str>,
}

pub async fn find_administrator_mfa_secret(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<Option<AdministratorMfaSecretRecord>, AppError> {
    sqlx::query_as::<_, AdministratorMfaSecretRecord>(
        "SELECT email, mfa_secret_ciphertext AS encrypted_secret
        FROM administrators
        WHERE id = $1 AND status = 'active' AND mfa_enabled_at IS NOT NULL",
    )
    .bind(administrator_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn regenerate_administrator_recovery_codes(
    pool: &PgPool,
    administrator_id: Uuid,
    session_id: Uuid,
    totp_step: u64,
    recovery_code_hashes: &[String],
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let session =
        sqlx::query_as::<_, (bool, Option<i64>, Option<chrono::DateTime<Utc>>, bool, bool)>(
            "SELECT administrator.status = 'active' AND administrator.mfa_enabled_at IS NOT NULL,
            administrator.last_totp_step, administrator.mfa_locked_until,
            session.revoked_at IS NULL AND session.expires_at > now()
                AND session.last_activity_at > now() - ($3 * interval '1 minute'),
            session.created_at >= now() - ($4 * interval '1 minute')
        FROM administrator_sessions session
        JOIN administrators administrator ON administrator.id = session.administrator_id
        WHERE session.id = $1 AND session.administrator_id = $2
        FOR UPDATE OF session, administrator",
        )
        .bind(session_id)
        .bind(administrator_id)
        .bind(ADMIN_SESSION_IDLE_TIMEOUT_MINUTES as f64)
        .bind(ADMIN_SENSITIVE_OPERATION_RECENT_AUTH_MINUTES as f64)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidSession)?;
    let (
        administrator_is_active,
        last_totp_step,
        mfa_locked_until,
        session_is_active,
        session_is_recent,
    ) = session;
    if !administrator_is_active || !session_is_active {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidSession.into());
    }
    if !session_is_recent {
        return Err(
            crate::enums::admin_auth_error::AdminAuthError::RecentAdministratorAuthenticationRequired
                .into(),
        );
    }
    if mfa_locked_until.is_some_and(|locked_until| locked_until > Utc::now())
        || last_totp_step.is_some_and(|last_step| last_step >= totp_step as i64)
    {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidMfaCode.into());
    }
    sqlx::query(
        "UPDATE administrators SET last_totp_step = $2,
            failed_mfa_attempts = 0, mfa_locked_until = NULL, updated_at = now()
        WHERE id = $1 AND status = 'active' AND mfa_enabled_at IS NOT NULL
            AND (last_totp_step IS NULL OR last_totp_step < $2)",
    )
    .bind(administrator_id)
    .bind(totp_step as i64)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE administrator_recovery_codes SET revoked_at = now()
        WHERE administrator_id = $1 AND used_at IS NULL AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    for recovery_code_hash in recovery_code_hashes {
        sqlx::query(
            "INSERT INTO administrator_recovery_codes (id, administrator_id, token_hash)
            VALUES ($1, $2, $3)",
        )
        .bind(Uuid::new_v4())
        .bind(administrator_id)
        .bind(recovery_code_hash)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: ADMINISTRATOR_RECOVERY_CODES_REGENERATED_AUDIT_ACTION,
            target_type: "administrator",
            target_id: administrator_id,
            result: "success",
            reason: Some("Administrator verified current authenticator code"),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn complete_administrator_mfa_and_create_session(
    pool: &PgPool,
    input: AdministratorMfaSessionCreation<'_>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let challenge = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<i64>)>(
        "SELECT challenge.administrator_id, challenge.purpose, challenge.encrypted_secret,
            administrator.last_totp_step
        FROM administrator_mfa_challenges challenge
        JOIN administrators administrator ON administrator.id = challenge.administrator_id
        WHERE challenge.token_hash = $1 AND challenge.consumed_at IS NULL
            AND challenge.expires_at > now() AND challenge.failed_attempts < 5
            AND administrator.status = 'active'
            AND (administrator.mfa_locked_until IS NULL OR administrator.mfa_locked_until <= now())
        FOR UPDATE OF challenge, administrator",
    )
    .bind(input.challenge_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge)?;
    let (administrator_id, purpose, challenge_secret, previous_step) = challenge;
    if previous_step.is_some_and(|previous| previous >= input.totp_step as i64) {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidMfaCode.into());
    }
    match AdministratorMfaChallengePurpose::try_from(purpose.as_str())? {
        AdministratorMfaChallengePurpose::Enrollment => {
            let secret = input
                .enrolled_secret
                .filter(|secret| challenge_secret.as_deref() == Some(*secret))
                .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge)?;
            let updated = sqlx::query(
                "UPDATE administrators SET mfa_secret_ciphertext = $2,
                    mfa_enabled_at = now(), last_totp_step = $3,
                    failed_mfa_attempts = 0, mfa_locked_until = NULL, updated_at = now()
                WHERE id = $1 AND status = 'active' AND mfa_enabled_at IS NULL",
            )
            .bind(administrator_id)
            .bind(secret)
            .bind(input.totp_step as i64)
            .execute(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?;
            if updated.rows_affected() != 1 {
                return Err(
                    crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge.into(),
                );
            }
            insert_administrative_audit_event(
                &mut transaction,
                AdministrativeAuditEvent {
                    actor_id: administrator_id,
                    action: "administrator.mfa_enrolled",
                    target_type: "administrator",
                    target_id: administrator_id,
                    result: "success",
                    reason: None,
                    correlation_id: input.correlation_id,
                },
            )
            .await?;
            for recovery_code_hash in input.recovery_code_hashes {
                sqlx::query(
                    "INSERT INTO administrator_recovery_codes (id, administrator_id, token_hash)
                    VALUES ($1, $2, $3)",
                )
                .bind(Uuid::new_v4())
                .bind(administrator_id)
                .bind(recovery_code_hash)
                .execute(&mut *transaction)
                .await
                .map_err(|_| AppError::Internal)?;
            }
        }
        AdministratorMfaChallengePurpose::Login => {
            let updated = sqlx::query(
                "UPDATE administrators SET last_totp_step = $2,
                    failed_mfa_attempts = 0, mfa_locked_until = NULL, updated_at = now()
                WHERE id = $1 AND status = 'active' AND mfa_enabled_at IS NOT NULL
                    AND (last_totp_step IS NULL OR last_totp_step < $2)",
            )
            .bind(administrator_id)
            .bind(input.totp_step as i64)
            .execute(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?;
            if updated.rows_affected() != 1 {
                return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidMfaCode.into());
            }
            insert_administrative_audit_event(
                &mut transaction,
                AdministrativeAuditEvent {
                    actor_id: administrator_id,
                    action: "administrator.login_succeeded",
                    target_type: "administrator",
                    target_id: administrator_id,
                    result: "success",
                    reason: None,
                    correlation_id: input.correlation_id,
                },
            )
            .await?;
        }
    }
    sqlx::query(
        "UPDATE administrator_mfa_challenges SET consumed_at = now()
        WHERE token_hash = $1 AND consumed_at IS NULL",
    )
    .bind(input.challenge_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let expiration = Utc::now() + Duration::hours(ADMIN_SESSION_LIFETIME_HOURS);
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(input.session_id)
    .bind(administrator_id)
    .bind(input.session_token_hash)
    .bind(expiration)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn consume_administrator_recovery_code_and_create_session(
    pool: &PgPool,
    challenge_hash: &str,
    recovery_code_hash: &str,
    session_id: Uuid,
    session_token_hash: &str,
    correlation_id: Option<&str>,
) -> Result<bool, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let challenge = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT challenge.administrator_id, challenge.purpose
        FROM administrator_mfa_challenges challenge
        JOIN administrators administrator ON administrator.id = challenge.administrator_id
        WHERE challenge.token_hash = $1 AND challenge.consumed_at IS NULL
            AND challenge.expires_at > now() AND challenge.failed_attempts < 5
            AND administrator.status = 'active'
            AND administrator.mfa_enabled_at IS NOT NULL
            AND (administrator.mfa_locked_until IS NULL OR administrator.mfa_locked_until <= now())
        FOR UPDATE OF challenge, administrator",
    )
    .bind(challenge_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some((administrator_id, purpose)) = challenge else {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge.into());
    };
    if AdministratorMfaChallengePurpose::try_from(purpose.as_str())?
        != AdministratorMfaChallengePurpose::Login
    {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge.into());
    }
    let used_code = sqlx::query(
        "UPDATE administrator_recovery_codes SET used_at = now()
        WHERE administrator_id = $1 AND token_hash = $2
            AND used_at IS NULL AND revoked_at IS NULL",
    )
    .bind(administrator_id)
    .bind(recovery_code_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .rows_affected()
        == 1;
    if !used_code {
        transaction
            .rollback()
            .await
            .map_err(|_| AppError::Internal)?;
        record_failed_administrator_mfa_attempt(pool, challenge_hash).await?;
        return Ok(false);
    }
    sqlx::query(
        "UPDATE administrators SET failed_mfa_attempts = 0, mfa_locked_until = NULL,
            updated_at = now() WHERE id = $1",
    )
    .bind(administrator_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE administrator_mfa_challenges SET consumed_at = now()
        WHERE token_hash = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, now() + ($4 * interval '1 hour'))",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(session_token_hash)
    .bind(ADMIN_SESSION_LIFETIME_HOURS as f64)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.mfa_recovery_succeeded",
            target_type: "administrator",
            target_id: administrator_id,
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(true)
}
