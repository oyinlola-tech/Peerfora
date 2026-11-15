use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::research::MAX_RESEARCH_TASK_SUBMISSIONS,
    enums::research::{ResearchTaskError, ResearchTaskStatus},
    errors::app_error::AppError,
    models::research_task::ResearchTask,
};

use super::{access, queries};

pub async fn submit_research_task_result(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
    result: &str,
) -> Result<ResearchTask, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id =
        access::find_active_human_actor_for_account(&mut transaction, account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    access::ensure_active_project_member(&mut transaction, task.project_id, actor_id).await?;
    if task.assigned_actor_id != Some(actor_id) {
        return Err(ResearchTaskError::AssignmentUnavailable.into());
    }
    if task.task_status != ResearchTaskStatus::Running.as_database_value() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    let next_submission_number: i16 = sqlx::query_scalar(
        "SELECT (COALESCE(MAX(submission_number), 0) + 1)::smallint
        FROM research_task_submissions WHERE task_id = $1",
    )
    .bind(task_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if next_submission_number > MAX_RESEARCH_TASK_SUBMISSIONS {
        return Err(ResearchTaskError::SubmissionLimitReached.into());
    }

    sqlx::query(
        "INSERT INTO research_task_submissions (
            task_id, submission_number, submitted_by_actor_id, result
        ) VALUES ($1, $2, $3, $4)",
    )
    .bind(task_id)
    .bind(next_submission_number)
    .bind(actor_id)
    .bind(result)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query("UPDATE research_tasks SET task_status = $2, updated_at = now() WHERE id = $1")
        .bind(task_id)
        .bind(ResearchTaskStatus::AwaitingReview.as_database_value())
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    queries::find_research_task_for_member(pool, task_id, account_id)
        .await?
        .ok_or(AppError::Internal)
}
