use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::repositories::{
    admin_role_assignment_repository::validate_role_key_is_delegable,
    admin_role_repository::lock_role_authorization,
};
use crate::{
    constants::admin_auth::{
        ADMIN_INVITATION_LIFETIME_HOURS, ADMIN_SENSITIVE_OPERATION_RECENT_AUTH_MINUTES,
        SUPER_ADMINISTRATOR_ROLE_KEY,
    },
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    models::admin_invitation::NewAdministratorInvitation,
    repositories::admin_auth_repository::{
        AdministrativeAuditEvent, insert_administrative_audit_event,
    },
};

pub async fn create_administrator_invitation(
    pool: &PgPool,
    invitation: NewAdministratorInvitation<'_>,
    inviter_session_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<String, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_role_authorization(&mut transaction).await?;
    ensure_inviter_is_active(&mut transaction, invitation.inviter_id).await?;
    let role_display_name =
        find_active_role_display_name(&mut transaction, invitation.role_key).await?;
    if invitation.role_key == SUPER_ADMINISTRATOR_ROLE_KEY {
        ensure_recent_administrator_authentication(
            &mut transaction,
            invitation.inviter_id,
            inviter_session_id,
        )
        .await?;
    }
    validate_role_key_is_delegable(&mut transaction, invitation.inviter_id, invitation.role_key)
        .await?;
    lock_administrator_invitation_email(&mut transaction, invitation.email).await?;
    reject_existing_administrator_email(&mut transaction, invitation.email).await?;
    revoke_pending_administrator_invitations(&mut transaction, invitation.email).await?;
    insert_administrator_invitation(
        &mut transaction,
        invitation.id,
        invitation.inviter_id,
        invitation.email,
        invitation.role_key,
        invitation.token_hash,
    )
    .await?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: invitation.inviter_id,
            action: "administrator.invitation_created",
            target_type: "administrator_invitation",
            target_id: invitation.id,
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(role_display_name)
}

async fn ensure_recent_administrator_authentication(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    session_id: Uuid,
) -> Result<(), AppError> {
    let recently_authenticated: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_sessions
            WHERE id = $1 AND administrator_id = $2
                AND created_at >= now() - ($3 * interval '1 minute')
                AND expires_at > now() AND revoked_at IS NULL
        )",
    )
    .bind(session_id)
    .bind(administrator_id)
    .bind(ADMIN_SENSITIVE_OPERATION_RECENT_AUTH_MINUTES as f64)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if recently_authenticated {
        Ok(())
    } else {
        Err(AdminAuthError::RecentAdministratorAuthenticationRequired.into())
    }
}

async fn ensure_inviter_is_active(
    transaction: &mut Transaction<'_, Postgres>,
    inviter_id: Uuid,
) -> Result<(), AppError> {
    let inviter_is_active: bool =
        sqlx::query_scalar("SELECT status = 'active' FROM administrators WHERE id = $1 FOR SHARE")
            .bind(inviter_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?
            .unwrap_or(false);
    if inviter_is_active {
        Ok(())
    } else {
        Err(AdminAuthError::PermissionDenied.into())
    }
}

async fn find_active_role_display_name(
    transaction: &mut Transaction<'_, Postgres>,
    role_key: &str,
) -> Result<String, AppError> {
    sqlx::query_scalar("SELECT display_name FROM roles WHERE role_key = $1 AND is_active FOR SHARE")
        .bind(role_key)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or_else(|| AdminAuthError::InvalidAdministratorRole.into())
}

async fn lock_administrator_invitation_email(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str,
) -> Result<(), AppError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended(lower($1), 118273))")
        .bind(email)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn reject_existing_administrator_email(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str,
) -> Result<(), AppError> {
    let account_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM administrators WHERE lower(administrators.email) = lower($1))",
    )
    .bind(email)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if account_exists {
        return Err(
            crate::enums::admin_auth_error::AdminAuthError::InvitationEmailAlreadyInUse.into(),
        );
    }
    Ok(())
}

async fn revoke_pending_administrator_invitations(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE administrator_invitations SET revoked_at = now()
        WHERE lower(email) = lower($1) AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(email)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn insert_administrator_invitation(
    transaction: &mut Transaction<'_, Postgres>,
    invitation_id: Uuid,
    inviter_id: Uuid,
    email: &str,
    role_key: &str,
    token_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrator_invitations
            (id, email, role_key, invited_by, token_hash, expires_at)
        VALUES ($1, $2, $3, $4, $5, now() + ($6 * interval '1 hour'))",
    )
    .bind(invitation_id)
    .bind(email)
    .bind(role_key)
    .bind(inviter_id)
    .bind(token_hash)
    .bind(ADMIN_INVITATION_LIFETIME_HOURS as f64)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn revoke_administrator_invitation_after_email_failure(
    pool: &PgPool,
    invitation_id: Uuid,
    inviter_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE administrator_invitations SET revoked_at = now()
        WHERE id = $1 AND invited_by = $2 AND consumed_at IS NULL AND revoked_at IS NULL",
    )
    .bind(invitation_id)
    .bind(inviter_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: inviter_id,
            action: "administrator.invitation_email_failed",
            target_type: "administrator_invitation",
            target_id: invitation_id,
            result: "failure",
            reason: None,
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn accept_administrator_invitation(
    pool: &PgPool,
    invitation_token_hash: &str,
    administrator_id: Uuid,
    password_hash: &str,
    correlation_id: Option<&str>,
) -> Result<(Uuid, String), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_role_authorization(&mut transaction).await?;
    let invitation = sqlx::query_as::<_, (Uuid, String, String, Uuid)>(
        "SELECT id, email, role_key, invited_by
        FROM administrator_invitations
        WHERE token_hash = $1 AND consumed_at IS NULL AND revoked_at IS NULL
            AND expires_at > now()
        FOR UPDATE",
    )
    .bind(invitation_token_hash)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(crate::enums::admin_auth_error::AdminAuthError::InvalidInvitation)?;
    let (invitation_id, email, role_key, inviter_id) = invitation;
    let inviter_is_active: bool =
        sqlx::query_scalar("SELECT status = 'active' FROM administrators WHERE id = $1 FOR SHARE")
            .bind(inviter_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?
            .unwrap_or(false);
    let role_is_active: bool =
        sqlx::query_scalar("SELECT is_active FROM roles WHERE role_key = $1 FOR SHARE")
            .bind(&role_key)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?
            .unwrap_or(false);
    if !inviter_is_active || !role_is_active {
        return Err(AdminAuthError::InvalidInvitation.into());
    }
    validate_role_key_is_delegable(&mut transaction, inviter_id, &role_key)
        .await
        .map_err(|_| AdminAuthError::InvalidInvitation)?;
    reject_existing_administrator_email(&mut transaction, &email).await?;
    insert_invited_administrator(
        &mut transaction,
        administrator_id,
        inviter_id,
        &email,
        password_hash,
    )
    .await?;
    grant_invited_administrator_role(&mut transaction, administrator_id, &role_key, inviter_id)
        .await?;
    consume_administrator_invitation(&mut transaction, invitation_id).await?;
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: "administrator.invitation_accepted",
            target_type: "administrator",
            target_id: administrator_id,
            result: "success",
            reason: Some("Administrator identity activated from a valid invitation"),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok((administrator_id, role_key))
}

async fn insert_invited_administrator(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    inviter_id: Uuid,
    email: &str,
    password_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrators (id, email, password_hash, created_by)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(administrator_id)
    .bind(email)
    .bind(password_hash)
    .bind(inviter_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn grant_invited_administrator_role(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    role_key: &str,
    inviter_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrator_roles (id, administrator_id, role_key, granted_by)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(role_key)
    .bind(inviter_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn consume_administrator_invitation(
    transaction: &mut Transaction<'_, Postgres>,
    invitation_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE administrator_invitations SET consumed_at = now()
        WHERE id = $1 AND consumed_at IS NULL AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(invitation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() != 1 {
        return Err(crate::enums::admin_auth_error::AdminAuthError::InvalidInvitation.into());
    }
    Ok(())
}
