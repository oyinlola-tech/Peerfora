use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::app_error::AppError;

mod mfa_reset;
mod session_revocation;
mod status_transition;

use status_transition::{AdministratorLifecycleChange, change_administrator_status};

pub use mfa_reset::reset_administrator_mfa;
pub use session_revocation::revoke_administrator_sessions;

pub async fn suspend_administrator(
    pool: &PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    change_administrator_status(
        pool,
        actor_id,
        session_id,
        target_id,
        reason,
        correlation_id,
        AdministratorLifecycleChange::Suspend,
    )
    .await
}

pub async fn reinstate_administrator(
    pool: &PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    change_administrator_status(
        pool,
        actor_id,
        session_id,
        target_id,
        reason,
        correlation_id,
        AdministratorLifecycleChange::Reinstate,
    )
    .await
}

pub async fn delete_administrator(
    pool: &PgPool,
    actor_id: Uuid,
    session_id: Uuid,
    target_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    change_administrator_status(
        pool,
        actor_id,
        session_id,
        target_id,
        reason,
        correlation_id,
        AdministratorLifecycleChange::Delete,
    )
    .await
}
