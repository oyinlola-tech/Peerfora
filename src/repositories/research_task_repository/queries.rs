use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::research::{DEFAULT_RESEARCH_TASK_LIST_LIMIT, MAX_RESEARCH_TASK_LIST_LIMIT},
    enums::research::ResearchTaskError,
    errors::app_error::AppError,
    models::research_task::ResearchTask,
};

const RESEARCH_TASK_FIELDS: &str =
    "SELECT task.id, task.project_id, creator.username AS created_by_username,
        task.title, task.objective, task.task_scope, task.allowed_data_sources,
        task.permitted_tools, task.expected_deliverables, task.max_runtime_seconds,
        task.max_result_bytes, task.task_status, assignee.username AS assigned_username,
        task.assignment_accepted_at, latest.submission_number AS latest_submission_number,
        latest.result AS latest_result, submitter.username AS latest_submission_by_username,
        latest.review_status AS latest_review_status, reviewer.username AS latest_reviewed_by_username,
        latest.reviewer_notes AS latest_review_notes, task.created_at, task.updated_at
    FROM research_tasks task
    JOIN human_actor_identities creator_identity
        ON creator_identity.actor_id = task.created_by_actor_id
    JOIN accounts creator ON creator.id = creator_identity.account_id
    LEFT JOIN human_actor_identities assignee_identity
        ON assignee_identity.actor_id = task.assigned_actor_id
    LEFT JOIN accounts assignee ON assignee.id = assignee_identity.account_id
    LEFT JOIN LATERAL (
        SELECT submission_number, submitted_by_actor_id, result, review_status,
            reviewed_by_actor_id, reviewer_notes
        FROM research_task_submissions
        WHERE task_id = task.id ORDER BY submission_number DESC LIMIT 1
    ) latest ON TRUE
    LEFT JOIN human_actor_identities submitter_identity
        ON submitter_identity.actor_id = latest.submitted_by_actor_id
    LEFT JOIN accounts submitter ON submitter.id = submitter_identity.account_id
    LEFT JOIN human_actor_identities reviewer_identity
        ON reviewer_identity.actor_id = latest.reviewed_by_actor_id
    LEFT JOIN accounts reviewer ON reviewer.id = reviewer_identity.account_id";

pub async fn find_research_task_for_member(
    pool: &PgPool,
    task_id: Uuid,
    account_id: Uuid,
) -> Result<Option<ResearchTask>, AppError> {
    let query = format!(
        "{RESEARCH_TASK_FIELDS}
        JOIN research_projects project ON project.id = task.project_id
        JOIN human_actor_identities member_identity ON member_identity.account_id = $2
        JOIN research_project_members member ON member.actor_id = member_identity.actor_id
            AND member.project_id = task.project_id AND member.membership_status = 'active'
        WHERE task.id = $1 AND project.project_status = 'active'"
    );
    sqlx::query_as::<_, ResearchTask>(&query)
        .bind(task_id)
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn list_research_tasks_for_member(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    requested_limit: Option<i64>,
) -> Result<Vec<ResearchTask>, AppError> {
    let limit = requested_limit
        .unwrap_or(DEFAULT_RESEARCH_TASK_LIST_LIMIT)
        .clamp(1, MAX_RESEARCH_TASK_LIST_LIMIT);
    let query = format!(
        "{RESEARCH_TASK_FIELDS}
        JOIN research_projects project ON project.id = task.project_id
        JOIN human_actor_identities member_identity ON member_identity.account_id = $2
        JOIN research_project_members member ON member.actor_id = member_identity.actor_id
            AND member.project_id = task.project_id AND member.membership_status = 'active'
        WHERE task.project_id = $1 AND project.project_status = 'active'
        ORDER BY task.updated_at DESC, task.id LIMIT $3"
    );
    sqlx::query_as::<_, ResearchTask>(&query)
        .bind(project_id)
        .bind(account_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn require_active_project_member(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let authorized: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM research_project_members member
            JOIN human_actor_identities identity ON identity.actor_id = member.actor_id
            JOIN research_projects project ON project.id = member.project_id
            WHERE member.project_id = $1 AND identity.account_id = $2
                AND member.membership_status = 'active' AND project.project_status = 'active'
        )",
    )
    .bind(project_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if authorized {
        Ok(())
    } else {
        Err(ResearchTaskError::ProjectNotFound.into())
    }
}
