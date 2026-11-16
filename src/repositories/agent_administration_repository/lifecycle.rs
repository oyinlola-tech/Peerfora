use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::{admin_auth, agents as agent_constants},
    enums::{
        admin_auth_error::AdminAuthError,
        agent::{AgentLifecycleAction, AgentReviewStatus},
        agent_error::AgentError,
        research::ResearchTaskStatus,
    },
    errors::app_error::AppError,
};

pub async fn administrate_agent_lifecycle(
    pool: &PgPool,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    action: AgentLifecycleAction,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_authorization(&mut transaction).await?;
    ensure_agent_management_permission(&mut transaction, administrator_id).await?;
    let status = lock_agent_status(&mut transaction, agent_actor_id).await?;
    let next_status = action.next_status(status)?;
    update_agent_status(&mut transaction, agent_actor_id, next_status).await?;
    revoke_agent_credentials(&mut transaction, agent_actor_id).await?;
    release_active_agent_tasks(&mut transaction, agent_actor_id).await?;
    insert_lifecycle_audit_record(
        &mut transaction,
        agent_actor_id,
        administrator_id,
        action,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

impl AgentLifecycleAction {
    fn next_status(self, current_status: AgentReviewStatus) -> Result<AgentReviewStatus, AppError> {
        let next_status = match (self, current_status) {
            (Self::Suspend, AgentReviewStatus::Approved | AgentReviewStatus::Active) => {
                AgentReviewStatus::Suspended
            }
            (Self::Reinstate, AgentReviewStatus::Suspended) => AgentReviewStatus::Approved,
            (
                Self::Revoke,
                AgentReviewStatus::Approved
                | AgentReviewStatus::Active
                | AgentReviewStatus::Suspended,
            ) => AgentReviewStatus::Revoked,
            _ => return Err(AgentError::AgentLifecycleConflict.into()),
        };
        Ok(next_status)
    }

    fn audit_action(self) -> &'static str {
        match self {
            Self::Suspend => agent_constants::AGENT_SUSPENDED_AUDIT_ACTION,
            Self::Reinstate => agent_constants::AGENT_REINSTATED_AUDIT_ACTION,
            Self::Revoke => agent_constants::AGENT_REVOKED_AUDIT_ACTION,
        }
    }
}

pub(super) async fn lock_administrator_authorization(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), AppError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(admin_auth::ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub(super) async fn ensure_agent_management_permission(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
) -> Result<(), AppError> {
    let permitted: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles assignment
            JOIN administrators administrator
                ON administrator.id = assignment.administrator_id
                AND administrator.status = $2
            JOIN roles role
                ON role.role_key = assignment.role_key AND role.is_active
            JOIN role_permissions permission
                ON permission.role_key = assignment.role_key
            WHERE assignment.administrator_id = $1
                AND assignment.revoked_at IS NULL
                AND permission.permission_key = $3
        )",
    )
    .bind(administrator_id)
    .bind(crate::enums::admin_auth::AdministratorLifecycleStatus::Active.as_database_value())
    .bind(agent_constants::ADMIN_AGENTS_MANAGE_PERMISSION)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if permitted {
        Ok(())
    } else {
        Err(AdminAuthError::PermissionDenied.into())
    }
}

async fn lock_agent_status(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<AgentReviewStatus, AppError> {
    let status: Option<String> = sqlx::query_scalar(
        "SELECT review_status FROM agent_actor_identities
        WHERE actor_id = $1 FOR UPDATE",
    )
    .bind(agent_actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let status = status.ok_or(AgentError::AgentNotFound)?;
    AgentReviewStatus::from_database_value(&status).ok_or(AppError::Internal)
}

async fn update_agent_status(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    status: AgentReviewStatus,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE agent_actor_identities SET review_status = $2, updated_at = now()
        WHERE actor_id = $1",
    )
    .bind(agent_actor_id)
    .bind(status.as_database_value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn revoke_agent_credentials(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE agent_machine_credentials SET revoked_at = now()
        WHERE agent_actor_id = $1 AND revoked_at IS NULL",
    )
    .bind(agent_actor_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn release_active_agent_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE research_tasks SET task_status = $2, assigned_actor_id = NULL,
            assignment_accepted_at = NULL, updated_at = now()
        WHERE assigned_actor_id = $1 AND task_status = ANY($3::text[])",
    )
    .bind(agent_actor_id)
    .bind(ResearchTaskStatus::Queued.as_database_value())
    .bind([
        ResearchTaskStatus::Assigned.as_database_value(),
        ResearchTaskStatus::Running.as_database_value(),
    ])
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn insert_lifecycle_audit_record(
    transaction: &mut Transaction<'_, Postgres>,
    agent_actor_id: Uuid,
    administrator_id: Uuid,
    action: AgentLifecycleAction,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id,
             result, reason, correlation_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(action.audit_action())
    .bind(agent_constants::AGENT_LIFECYCLE_AUDIT_TARGET_TYPE)
    .bind(agent_actor_id)
    .bind(admin_auth::ADMIN_AUDIT_SUCCESS_RESULT)
    .bind(reason)
    .bind(correlation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
