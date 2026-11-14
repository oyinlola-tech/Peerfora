use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::admin_auth::ADMIN_ACCOUNT_SESSIONS_REVOKED_AUDIT_ACTION,
    enums::{account_lifecycle::AccountLifecycleStatus, admin_auth_error::AdminAuthError},
    errors::app_error::AppError,
    repositories::admin_auth_repository::{
        AdministrativeAuditEvent, insert_administrative_audit_event,
    },
};

pub async fn change_account_lifecycle_status(
    pool: &PgPool,
    account_id: Uuid,
    administrator_id: Uuid,
    expected_status: AccountLifecycleStatus,
    new_status: AccountLifecycleStatus,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let current_status = lock_account_for_administration(&mut transaction, account_id).await?;
    let current_status = AccountLifecycleStatus::try_from(current_status.as_str())
        .map_err(|_| AppError::Internal)?;
    if current_status != expected_status {
        return Err(AdminAuthError::AccountStateConflict.into());
    }
    if new_status == AccountLifecycleStatus::Suspended {
        reject_suspension_when_account_owns_communities(&mut transaction, account_id).await?;
    }
    update_account_lifecycle_status(&mut transaction, account_id, new_status).await?;
    if new_status == AccountLifecycleStatus::Suspended {
        revoke_account_access_tokens(&mut transaction, account_id).await?;
    }
    let action = match new_status {
        AccountLifecycleStatus::Active => "account.reinstated",
        AccountLifecycleStatus::Suspended => "account.suspended",
        AccountLifecycleStatus::PendingDeletion | AccountLifecycleStatus::Deleted => {
            return Err(AppError::Internal);
        }
    };
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action,
            target_type: "account",
            target_id: account_id,
            result: "success",
            reason: Some(reason_code),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn revoke_account_sessions_for_administration(
    pool: &PgPool,
    account_id: Uuid,
    administrator_id: Uuid,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<u64, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_account_for_administration(&mut transaction, account_id).await?;
    let revoked_sessions = sqlx::query("DELETE FROM account_sessions WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?
        .rows_affected();
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: ADMIN_ACCOUNT_SESSIONS_REVOKED_AUDIT_ACTION,
            target_type: "account",
            target_id: account_id,
            result: "success",
            reason: Some(reason_code),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(revoked_sessions)
}

async fn reject_suspension_when_account_owns_communities(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<(), AppError> {
    let account_owns_communities: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM communities WHERE owner_account_id = $1)")
            .bind(account_id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    if account_owns_communities {
        return Err(AdminAuthError::AccountOwnsCommunities.into());
    }
    Ok(())
}

async fn lock_account_for_administration(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<String, AppError> {
    sqlx::query_scalar::<_, String>("SELECT account_status FROM accounts WHERE id = $1 FOR UPDATE")
        .bind(account_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(AdminAuthError::AccountNotFound.into())
}

async fn update_account_lifecycle_status(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    new_status: AccountLifecycleStatus,
) -> Result<(), AppError> {
    sqlx::query("UPDATE accounts SET account_status = $2, updated_at = now() WHERE id = $1")
        .bind(account_id)
        .bind(new_status.as_database_value())
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_account_access_tokens(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM account_sessions WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM password_reset_tokens WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query("DELETE FROM email_verification_tokens WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}
