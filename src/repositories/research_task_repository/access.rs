use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{enums::research::ResearchTaskError, errors::app_error::AppError};

pub struct LockedResearchTask {
    pub project_id: Uuid,
    pub created_by_actor_id: Uuid,
    pub task_status: String,
    pub assigned_actor_id: Option<Uuid>,
    pub assignment_accepted_at: Option<DateTime<Utc>>,
}

pub async fn find_active_human_actor_for_account(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT identity.actor_id
        FROM human_actor_identities identity
        JOIN accounts account ON account.id = identity.account_id
        WHERE account.id = $1 AND account.account_status = 'active'
            AND account.email_verified_at IS NOT NULL
        FOR UPDATE OF account",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchTaskError::PermissionDenied.into())
}

pub async fn lock_active_research_project(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<(), AppError> {
    let status: Option<String> =
        sqlx::query_scalar("SELECT project_status FROM research_projects WHERE id = $1 FOR UPDATE")
            .bind(project_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    match status.as_deref() {
        Some("active") => Ok(()),
        Some(_) => Err(ResearchTaskError::ProjectNotFound.into()),
        None => Err(ResearchTaskError::ProjectNotFound.into()),
    }
}

pub async fn lock_research_task(
    transaction: &mut Transaction<'_, Postgres>,
    task_id: Uuid,
) -> Result<LockedResearchTask, AppError> {
    let project_id: Option<Uuid> =
        sqlx::query_scalar("SELECT project_id FROM research_tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    let project_id = project_id.ok_or(ResearchTaskError::TaskNotFound)?;
    lock_active_research_project(transaction, project_id).await?;

    sqlx::query_as::<_, LockedResearchTaskRow>(
        "SELECT project_id, created_by_actor_id, task_status, assigned_actor_id,
            assignment_accepted_at
        FROM research_tasks WHERE id = $1 FOR UPDATE",
    )
    .bind(task_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .map(Into::into)
    .ok_or(ResearchTaskError::TaskNotFound.into())
}

#[derive(sqlx::FromRow)]
struct LockedResearchTaskRow {
    project_id: Uuid,
    created_by_actor_id: Uuid,
    task_status: String,
    assigned_actor_id: Option<Uuid>,
    assignment_accepted_at: Option<DateTime<Utc>>,
}

impl From<LockedResearchTaskRow> for LockedResearchTask {
    fn from(row: LockedResearchTaskRow) -> Self {
        Self {
            project_id: row.project_id,
            created_by_actor_id: row.created_by_actor_id,
            task_status: row.task_status,
            assigned_actor_id: row.assigned_actor_id,
            assignment_accepted_at: row.assignment_accepted_at,
        }
    }
}

pub async fn ensure_project_task_manager(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let authorized: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM research_project_members
            WHERE project_id = $1 AND actor_id = $2 AND membership_status = 'active'
                AND member_role IN ('owner', 'supervisor', 'researcher')
        )",
    )
    .bind(project_id)
    .bind(actor_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if authorized {
        Ok(())
    } else {
        Err(ResearchTaskError::PermissionDenied.into())
    }
}

pub async fn ensure_active_project_member(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let authorized: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM research_project_members
            WHERE project_id = $1 AND actor_id = $2 AND membership_status = 'active'
        )",
    )
    .bind(project_id)
    .bind(actor_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if authorized {
        Ok(())
    } else {
        Err(ResearchTaskError::PermissionDenied.into())
    }
}
