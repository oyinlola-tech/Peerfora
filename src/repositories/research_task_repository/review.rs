use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::{ResearchTaskError, ResearchTaskReviewDecision, ResearchTaskStatus},
    errors::app_error::AppError,
};

use super::access;

pub async fn review_research_task_submission(
    pool: &PgPool,
    task_id: Uuid,
    reviewer_account_id: Uuid,
    decision: ResearchTaskReviewDecision,
    review_notes: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let reviewer_actor_id =
        access::find_active_human_actor_for_account(&mut transaction, reviewer_account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    if task.task_status != ResearchTaskStatus::AwaitingReview.as_database_value() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    if task.assigned_actor_id == Some(reviewer_actor_id) {
        return Err(ResearchTaskError::SelfReviewForbidden.into());
    }
    ensure_research_task_reviewer(&mut transaction, task.project_id, reviewer_actor_id).await?;
    let pending_submission_id = find_pending_submission_id(&mut transaction, task_id).await?;
    let submission_author_id =
        find_submission_author_id(&mut transaction, pending_submission_id).await?;
    if submission_author_id == reviewer_actor_id {
        return Err(ResearchTaskError::SelfReviewForbidden.into());
    }

    sqlx::query(
        "UPDATE research_task_submissions SET review_status = $2,
            reviewed_by_actor_id = $3, reviewer_notes = $4, reviewed_at = now()
        WHERE id = $1",
    )
    .bind(pending_submission_id)
    .bind(decision.as_database_value())
    .bind(reviewer_actor_id)
    .bind(review_notes)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let task_status = task_status_after_review(decision);
    let accepted_at = if decision == ResearchTaskReviewDecision::RevisionRequested {
        None
    } else {
        task.assignment_accepted_at
    };
    sqlx::query(
        "UPDATE research_tasks SET task_status = $2, assignment_accepted_at = $3,
            updated_at = now() WHERE id = $1",
    )
    .bind(task_id)
    .bind(task_status.as_database_value())
    .bind(accepted_at)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn ensure_research_task_reviewer(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    reviewer_actor_id: Uuid,
) -> Result<(), AppError> {
    let authorized: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM research_project_members
            WHERE project_id = $1 AND actor_id = $2 AND membership_status = 'active'
                AND member_role IN ('owner', 'supervisor', 'reviewer')
        )",
    )
    .bind(project_id)
    .bind(reviewer_actor_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if authorized {
        Ok(())
    } else {
        Err(ResearchTaskError::PermissionDenied.into())
    }
}

async fn find_pending_submission_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT id FROM research_task_submissions
        WHERE task_id = $1 AND review_status = 'pending'
        ORDER BY submission_number DESC LIMIT 1",
    )
    .bind(task_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchTaskError::SubmissionUnavailable.into())
}

async fn find_submission_author_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    submission_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT submitted_by_actor_id FROM research_task_submissions WHERE id = $1 FOR UPDATE",
    )
    .bind(submission_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}

fn task_status_after_review(decision: ResearchTaskReviewDecision) -> ResearchTaskStatus {
    match decision {
        ResearchTaskReviewDecision::Approved => ResearchTaskStatus::Completed,
        ResearchTaskReviewDecision::RevisionRequested => ResearchTaskStatus::Assigned,
        ResearchTaskReviewDecision::Rejected => ResearchTaskStatus::Failed,
    }
}
