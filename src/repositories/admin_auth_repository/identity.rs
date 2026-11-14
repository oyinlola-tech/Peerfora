use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_CHALLENGE_LIFETIME_MINUTES, ADMIN_LOGIN_FAILED_ATTEMPT_LIMIT,
        ADMIN_LOGIN_LOCK_MINUTES, ADMIN_MFA_FAILED_ATTEMPT_LIMIT, ADMIN_MFA_LOCK_MINUTES,
        SEEDED_ROOT_ADMINISTRATOR_ID,
    },
    enums::admin_auth::AdministratorMfaChallengePurpose,
    errors::app_error::AppError,
    models::admin_identity::{AdministratorChallengeRecord, AdministratorLoginRecord},
};

use super::audit::{AdministrativeAuditEvent, insert_administrative_audit_event};

pub async fn bootstrap_first_administrator(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock(734125091)")
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    let root_id = Uuid::parse_str(SEEDED_ROOT_ADMINISTRATOR_ID).map_err(|_| AppError::Internal)?;
    let activated: bool = sqlx::query_scalar(
        "UPDATE administrators
        SET email = $1, password_hash = $2, status = 'active', updated_at = now()
        WHERE id = $3 AND is_seeded_root AND status = 'pending_activation'
        RETURNING TRUE",
    )
    .bind(email)
    .bind(password_hash)
    .bind(root_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| {
        let is_duplicate_email = error
            .as_database_error()
            .and_then(sqlx::error::DatabaseError::code)
            .is_some_and(|code| code == "23505");
        if is_duplicate_email {
            crate::enums::admin_auth_error::AdminAuthError::AdministratorEmailAlreadyInUse.into()
        } else {
            AppError::Internal
        }
    })?
    .unwrap_or(false);
    if !activated {
        return Err(
            crate::enums::admin_auth_error::AdminAuthError::BootstrapAlreadyCompleted.into(),
        );
    }
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: root_id,
            action: "administrator.bootstrap_completed",
            target_type: "administrator",
            target_id: root_id,
            result: "success",
            reason: Some("Migration-seeded root activated through bootstrap command"),
            correlation_id: None,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn find_administrator_for_login(
    pool: &PgPool,
    email: &str,
) -> Result<Option<AdministratorLoginRecord>, AppError> {
    sqlx::query_as::<_, AdministratorLoginRecord>(
        "SELECT id, email, password_hash, status = 'active'
            AND (login_locked_until IS NULL OR login_locked_until <= now()) AS is_active,
            mfa_secret_ciphertext,
            COALESCE(mfa_locked_until > now(), FALSE) AS mfa_locked
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
            administrator.status = 'active' AND
                (administrator.mfa_locked_until IS NULL OR administrator.mfa_locked_until <= now())
                AS is_active
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
    sqlx::query(
        "UPDATE administrators
        SET failed_mfa_attempts = CASE
                WHEN mfa_locked_until IS NOT NULL AND mfa_locked_until <= now() THEN 1
                ELSE LEAST(failed_mfa_attempts + 1, $2) END,
            mfa_locked_until = CASE
                WHEN mfa_locked_until IS NOT NULL AND mfa_locked_until <= now() THEN NULL
                WHEN (mfa_locked_until IS NULL OR mfa_locked_until <= now())
                    AND failed_mfa_attempts + 1 >= $2
                    THEN now() + ($3 * interval '1 minute')
                ELSE mfa_locked_until END,
            updated_at = now()
        WHERE id = $1",
    )
    .bind(administrator_id)
    .bind(ADMIN_MFA_FAILED_ATTEMPT_LIMIT)
    .bind(ADMIN_MFA_LOCK_MINUTES as f64)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action:
                crate::constants::admin_auth::ADMINISTRATOR_MFA_VERIFICATION_FAILED_AUDIT_ACTION,
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

pub async fn record_failed_administrator_mfa_for_identity(
    pool: &PgPool,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let is_active: bool =
        sqlx::query_scalar("SELECT status = 'active' FROM administrators WHERE id = $1 FOR UPDATE")
            .bind(administrator_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?
            .unwrap_or(false);
    if !is_active {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidSession.into());
    }
    sqlx::query(
        "UPDATE administrators SET
            failed_mfa_attempts = CASE
                WHEN mfa_locked_until IS NOT NULL AND mfa_locked_until <= now() THEN 1
                ELSE LEAST(failed_mfa_attempts + 1, $2) END,
            mfa_locked_until = CASE
                WHEN mfa_locked_until IS NOT NULL AND mfa_locked_until <= now() THEN NULL
                WHEN (mfa_locked_until IS NULL OR mfa_locked_until <= now())
                    AND failed_mfa_attempts + 1 >= $2
                    THEN now() + ($3 * interval '1 minute')
                ELSE mfa_locked_until END,
            updated_at = now()
        WHERE id = $1",
    )
    .bind(administrator_id)
    .bind(ADMIN_MFA_FAILED_ATTEMPT_LIMIT)
    .bind(ADMIN_MFA_LOCK_MINUTES as f64)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action:
                crate::constants::admin_auth::ADMINISTRATOR_MFA_VERIFICATION_FAILED_AUDIT_ACTION,
            target_type: "administrator",
            target_id: administrator_id,
            result: "failure",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}
