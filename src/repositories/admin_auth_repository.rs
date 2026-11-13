use chrono::{Duration, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_CHALLENGE_LIFETIME_MINUTES, ADMIN_LOGIN_FAILED_ATTEMPT_LIMIT,
        ADMIN_LOGIN_LOCK_MINUTES, ADMIN_MFA_FAILED_ATTEMPT_LIMIT,
        ADMIN_SESSION_IDLE_TIMEOUT_MINUTES, ADMIN_SESSION_LIFETIME_HOURS,
    },
    enums::admin_auth::AdministratorMfaChallengePurpose,
    errors::app_error::AppError,
    models::admin_identity::{
        AdministrativeAuditLogRecord, AdministratorChallengeRecord, AdministratorLoginRecord,
        AdministratorSessionIdentity, AdministratorSessionRecord,
    },
};

struct AdministrativeAuditEvent<'a> {
    actor_id: Uuid,
    action: &'a str,
    target_type: &'a str,
    target_id: Uuid,
    result: &'a str,
    reason: Option<&'a str>,
    correlation_id: Option<&'a str>,
}

pub async fn bootstrap_first_administrator(
    pool: &PgPool,
    administrator_id: Uuid,
    email: &str,
    password_hash: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock(734125091)")
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    let existing_count: i64 = sqlx::query_scalar("SELECT count(*) FROM administrators")
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    if existing_count != 0 {
        return Err(
            crate::enums::admin_auth_error::AdminAuthError::BootstrapAlreadyCompleted.into(),
        );
    }
    insert_administrator(&mut transaction, administrator_id, email, password_hash).await?;
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
        VALUES ($1, $2, 'super_administrator', $2)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.bootstrap_completed",
            target_type: "administrator",
            target_id: administrator_id,
            result: "success",
            reason: Some("Initial administrator provisioned through bootstrap command"),
            correlation_id: None,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn insert_administrator(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    email: &str,
    password_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash)
        VALUES ($1, $2, $3)",
    )
    .bind(administrator_id)
    .bind(email)
    .bind(password_hash)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn find_administrator_for_login(
    pool: &PgPool,
    email: &str,
) -> Result<Option<AdministratorLoginRecord>, AppError> {
    sqlx::query_as::<_, AdministratorLoginRecord>(
        "SELECT id, email, password_hash, status = 'active'
            AND (login_locked_until IS NULL OR login_locked_until <= now()) AS is_active,
            mfa_secret_ciphertext
        FROM administrators WHERE lower(email) = lower($1)",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn record_failed_administrator_login(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE administrators
        SET failed_login_attempts = LEAST(failed_login_attempts + 1, $2),
            login_locked_until = CASE
                WHEN failed_login_attempts + 1 >= $2
                    THEN now() + ($3 * interval '1 minute')
                ELSE login_locked_until END,
            updated_at = now()
        WHERE id = $1",
    )
    .bind(administrator_id)
    .bind(ADMIN_LOGIN_FAILED_ATTEMPT_LIMIT)
    .bind(ADMIN_LOGIN_LOCK_MINUTES as f64)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.login_failed",
            target_type: "administrator",
            target_id: administrator_id,
            result: "failure",
            reason: None,
            correlation_id: None,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn reset_administrator_login_failures(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrators SET failed_login_attempts = 0, login_locked_until = NULL
        WHERE id = $1",
    )
    .bind(administrator_id)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn create_administrator_mfa_challenge(
    pool: &PgPool,
    administrator_id: Uuid,
    challenge_hash: &str,
    purpose: AdministratorMfaChallengePurpose,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrator_mfa_challenges
            (token_hash, administrator_id, purpose, expires_at)
        VALUES ($1, $2, $3, now() + ($4 * interval '1 minute'))",
    )
    .bind(challenge_hash)
    .bind(administrator_id)
    .bind(purpose.as_database_value())
    .bind(ADMIN_CHALLENGE_LIFETIME_MINUTES as f64)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn find_active_administrator_challenge(
    pool: &PgPool,
    challenge_hash: &str,
) -> Result<Option<AdministratorChallengeRecord>, AppError> {
    sqlx::query_as::<_, AdministratorChallengeRecord>(
        "SELECT challenge.administrator_id, administrator.email, challenge.purpose,
            challenge.encrypted_secret,
            administrator.status = 'active' AS is_active
        FROM administrator_mfa_challenges challenge
        JOIN administrators administrator ON administrator.id = challenge.administrator_id
        WHERE challenge.token_hash = $1 AND challenge.consumed_at IS NULL
            AND challenge.expires_at > now() AND challenge.failed_attempts < 5",
    )
    .bind(challenge_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn store_pending_administrator_mfa_secret(
    pool: &PgPool,
    challenge_hash: &str,
    encrypted_secret: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE administrator_mfa_challenges
        SET encrypted_secret = $2
        WHERE token_hash = $1 AND purpose = 'enrollment' AND consumed_at IS NULL
            AND expires_at > now() AND failed_attempts < 5",
    )
    .bind(challenge_hash)
    .bind(encrypted_secret)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() != 1 {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge.into());
    }
    Ok(())
}

pub async fn record_failed_administrator_mfa_attempt(
    pool: &PgPool,
    challenge_hash: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let administrator_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT administrator_id FROM administrator_mfa_challenges
        WHERE token_hash = $1 AND consumed_at IS NULL FOR UPDATE",
    )
    .bind(challenge_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some(administrator_id) = administrator_id else {
        return transaction.commit().await.map_err(|_| AppError::Internal);
    };
    sqlx::query(
        "UPDATE administrator_mfa_challenges
        SET failed_attempts = LEAST(failed_attempts + 1, $2),
            consumed_at = CASE WHEN failed_attempts + 1 >= $2 THEN now() ELSE consumed_at END
        WHERE token_hash = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_hash)
    .bind(ADMIN_MFA_FAILED_ATTEMPT_LIMIT)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.mfa_verification_failed",
            target_type: "administrator",
            target_id: administrator_id,
            result: "failure",
            reason: None,
            correlation_id: None,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn complete_administrator_mfa_and_create_session(
    pool: &PgPool,
    challenge_hash: &str,
    totp_step: u64,
    enrolled_secret: Option<&str>,
    session_id: Uuid,
    session_token_hash: &str,
    correlation_id: Option<&str>,
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
        FOR UPDATE OF challenge, administrator",
    )
    .bind(challenge_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge)?;
    let (administrator_id, purpose, challenge_secret, previous_step) = challenge;
    if previous_step.is_some_and(|previous| previous >= totp_step as i64) {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidMfaCode.into());
    }
    match AdministratorMfaChallengePurpose::try_from(purpose.as_str())? {
        AdministratorMfaChallengePurpose::Enrollment => {
            let secret = enrolled_secret
                .filter(|secret| challenge_secret.as_deref() == Some(*secret))
                .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidChallenge)?;
            let updated = sqlx::query(
                "UPDATE administrators SET mfa_secret_ciphertext = $2,
                    mfa_enabled_at = now(), last_totp_step = $3, updated_at = now()
                WHERE id = $1 AND status = 'active' AND mfa_enabled_at IS NULL",
            )
            .bind(administrator_id)
            .bind(secret)
            .bind(totp_step as i64)
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
                    correlation_id,
                },
            )
            .await?;
        }
        AdministratorMfaChallengePurpose::Login => {
            let updated = sqlx::query(
                "UPDATE administrators SET last_totp_step = $2, updated_at = now()
                WHERE id = $1 AND status = 'active' AND mfa_enabled_at IS NOT NULL
                    AND (last_totp_step IS NULL OR last_totp_step < $2)",
            )
            .bind(administrator_id)
            .bind(totp_step as i64)
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
                    correlation_id,
                },
            )
            .await?;
        }
    }
    sqlx::query(
        "UPDATE administrator_mfa_challenges SET consumed_at = now()
        WHERE token_hash = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let expiration = Utc::now() + Duration::hours(ADMIN_SESSION_LIFETIME_HOURS);
    sqlx::query(
        "INSERT INTO administrator_sessions (id, administrator_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(session_token_hash)
    .bind(expiration)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn find_active_administrator_session(
    pool: &PgPool,
    session_token_hash: &str,
) -> Result<Option<AdministratorSessionIdentity>, AppError> {
    let identity = sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT session.id, session.administrator_id
        FROM administrator_sessions session
        JOIN administrators administrator ON administrator.id = session.administrator_id
        WHERE session.token_hash = $1 AND session.revoked_at IS NULL
            AND session.expires_at > now()
            AND session.last_activity_at > now() - ($2 * interval '1 minute')
            AND administrator.status = 'active' AND administrator.mfa_enabled_at IS NOT NULL",
    )
    .bind(session_token_hash)
    .bind(ADMIN_SESSION_IDLE_TIMEOUT_MINUTES as f64)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let Some((session_id, administrator_id)) = identity else {
        return Ok(None);
    };
    sqlx::query("UPDATE administrator_sessions SET last_activity_at = now() WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Some(AdministratorSessionIdentity {
        administrator_id,
        session_id,
    }))
}

pub async fn find_administrator_session_profile(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<(String, Vec<String>, Vec<String>), AppError> {
    sqlx::query_as::<_, (String, Vec<String>, Vec<String>)>(
        "SELECT administrator.email,
            ARRAY(SELECT role_key FROM administrator_roles
                WHERE administrator_roles.administrator_id = administrator.id),
            ARRAY(SELECT DISTINCT role_permissions.permission_key
                FROM administrator_roles
                JOIN role_permissions USING (role_key)
                WHERE administrator_roles.administrator_id = administrator.id)
        FROM administrators administrator WHERE administrator.id = $1",
    )
    .bind(administrator_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidSession.into())
}

pub async fn administrator_has_permission(
    pool: &PgPool,
    administrator_id: Uuid,
    permission_key: &str,
) -> Result<bool, AppError> {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles
            JOIN role_permissions USING (role_key)
            WHERE administrator_roles.administrator_id = $1
                AND role_permissions.permission_key = $2
        )",
    )
    .bind(administrator_id)
    .bind(permission_key)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn record_administrator_permission_denial(
    pool: &PgPool,
    administrator_id: Uuid,
    permission_key: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.permission_denied",
            target_type: "permission",
            target_id: Uuid::nil(),
            result: "failure",
            reason: Some(permission_key),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_administrative_audit_logs(
    pool: &PgPool,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AdministrativeAuditLogRecord>, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.audit_logs_viewed",
            target_type: "administrative_audit_logs",
            target_id: Uuid::nil(),
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    let records = sqlx::query_as::<_, AdministrativeAuditLogRecord>(
        "SELECT id, actor_administrator_id, action, target_type, target_id,
            result, reason, correlation_id, created_at
        FROM administrative_audit_logs
        ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(records)
}

pub async fn revoke_administrator_session(
    pool: &PgPool,
    administrator_id: Uuid,
    session_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let revoked = sqlx::query(
        "UPDATE administrator_sessions SET revoked_at = now()
        WHERE id = $1 AND administrator_id = $2 AND revoked_at IS NULL",
    )
    .bind(session_id)
    .bind(administrator_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if revoked.rows_affected() != 1 {
        return Err(crate::enums::admin_auth_error::AdminAuthError::SessionNotFound.into());
    }
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.session_revoked",
            target_type: "administrator_session",
            target_id: session_id,
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_administrator_sessions(
    pool: &PgPool,
    administrator_id: Uuid,
) -> Result<Vec<AdministratorSessionRecord>, AppError> {
    sqlx::query_as::<_, AdministratorSessionRecord>(
        "SELECT id, created_at, last_activity_at, expires_at, revoked_at
        FROM administrator_sessions WHERE administrator_id = $1
        ORDER BY created_at DESC, id DESC LIMIT 50",
    )
    .bind(administrator_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

async fn insert_administrative_audit_event(
    transaction: &mut Transaction<'_, Postgres>,
    event: AdministrativeAuditEvent<'_>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id,
                result, reason, correlation_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(Uuid::new_v4())
    .bind(event.actor_id)
    .bind(event.action)
    .bind(event.target_type)
    .bind(event.target_id)
    .bind(event.result)
    .bind(event.reason)
    .bind(event.correlation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
