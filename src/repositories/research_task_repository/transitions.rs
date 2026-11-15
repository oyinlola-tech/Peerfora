use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::{ResearchTaskError, ResearchTaskStatus},
    errors::app_error::AppError,
};

use super::access;

pub async fn queue_research_task(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id =
        access::find_active_human_actor_for_account(&mut transaction, account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    access::ensure_project_task_manager(&mut transaction, task.project_id, actor_id).await?;
    ensure_status(&task.task_status, ResearchTaskStatus::Draft)?;
    update_task_status(&mut transaction, task_id, ResearchTaskStatus::Queued).await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn start_research_task(
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
    ensure_status(&task.task_status, ResearchTaskStatus::Assigned)?;
    if task.assignment_accepted_at.is_none() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    update_task_status(&mut transaction, task_id, ResearchTaskStatus::Running).await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn cancel_research_task(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id =
        access::find_active_human_actor_for_account(&mut transaction, account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    if task.created_by_actor_id != actor_id {
        access::ensure_project_task_manager(&mut transaction, task.project_id, actor_id).await?;
    } else {
        access::ensure_active_project_member(&mut transaction, task.project_id, actor_id).await?;
    }
    if matches!(
        task.task_status.as_str(),
        "completed" | "failed" | "cancelled"
    ) {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    sqlx::query(
        "UPDATE research_tasks SET task_status = $2, cancelled_at = now(), updated_at = now()
        WHERE id = $1",
    )
    .bind(task_id)
    .bind(ResearchTaskStatus::Cancelled.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

fn ensure_status(
    current_status: &str,
    expected_status: ResearchTaskStatus,
) -> Result<(), AppError> {
    if current_status == expected_status.as_database_value() {
        Ok(())
    } else {
        Err(ResearchTaskError::InvalidTransition.into())
    }
}

async fn update_task_status(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task_id: Uuid,
    new_status: ResearchTaskStatus,
) -> Result<(), AppError> {
    sqlx::query("UPDATE research_tasks SET task_status = $2, updated_at = now() WHERE id = $1")
        .bind(task_id)
        .bind(new_status.as_database_value())
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}
