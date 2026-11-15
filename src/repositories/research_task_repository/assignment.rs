use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::research::{ResearchTaskError, ResearchTaskStatus},
    errors::app_error::AppError,
};

use super::access;

pub async fn assign_research_task(
    pool: &PgPool,
    task_id: Uuid,
    manager_account_id: Uuid,
    assignee_username: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let manager_actor_id =
        access::find_active_human_actor_for_account(&mut transaction, manager_account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    access::ensure_project_task_manager(&mut transaction, task.project_id, manager_actor_id)
        .await?;
    ensure_task_status(&task.task_status, ResearchTaskStatus::Queued)?;
    let assignee_actor_id =
        find_active_member_actor_by_username(&mut transaction, task.project_id, assignee_username)
            .await?;

    sqlx::query(
        "UPDATE research_tasks SET assigned_actor_id = $2, task_status = $3,
            assignment_accepted_at = NULL, updated_at = now()
        WHERE id = $1",
    )
    .bind(task_id)
    .bind(assignee_actor_id)
    .bind(ResearchTaskStatus::Assigned.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn accept_research_task_assignment(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id =
        access::find_active_human_actor_for_account(&mut transaction, account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    access::ensure_active_project_member(&mut transaction, task.project_id, actor_id).await?;
    if task.assigned_actor_id != Some(actor_id) {
        return Err(ResearchTaskError::AssignmentUnavailable.into());
    }
    ensure_task_status(&task.task_status, ResearchTaskStatus::Assigned)?;
    if task.assignment_accepted_at.is_some() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    sqlx::query(
        "UPDATE research_tasks SET assignment_accepted_at = now(), updated_at = now()
        WHERE id = $1",
    )
    .bind(task_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn find_active_member_actor_by_username(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    username: &str,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT identity.actor_id
        FROM human_actor_identities identity
        JOIN accounts account ON account.id = identity.account_id
        JOIN research_project_members member ON member.actor_id = identity.actor_id
        WHERE member.project_id = $1 AND lower(account.username) = lower($2)
            AND member.membership_status = 'active' AND member.member_role <> 'observer'
            AND account.account_status = 'active' AND account.email_verified_at IS NOT NULL",
    )
    .bind(project_id)
    .bind(username)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchTaskError::AssigneeUnavailable.into())
}

fn ensure_task_status(
    current_status: &str,
    expected_status: ResearchTaskStatus,
) -> Result<(), AppError> {
    if current_status == expected_status.as_database_value() {
        Ok(())
    } else {
        Err(ResearchTaskError::InvalidTransition.into())
    }
}
