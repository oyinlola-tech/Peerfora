use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::research::{
        MAX_RESEARCH_TASK_LIST_LIMIT, MAX_RESEARCH_TASK_SOURCE_REFERENCES,
        MAX_RESEARCH_TASK_SUBMISSIONS,
    },
    enums::{
        agent::{AgentCapability, AgentCapabilityReviewStatus, AgentReviewStatus},
        agent_error::AgentError,
        research::{ResearchProjectStatus, ResearchTaskError, ResearchTaskStatus},
    },
    errors::app_error::AppError,
    models::agent_credential::{AgentResearchTask, AgentResearchTaskSummary},
};

pub struct AgentTaskSource {
    pub title: String,
    pub url: String,
    pub doi: Option<String>,
}

async fn lock_approved_research_agent(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    let authorized: Option<Uuid> = sqlx::query_scalar(
        "SELECT agent.actor_id FROM agent_actor_identities agent
        JOIN agent_capability_requests capability ON capability.agent_actor_id = agent.actor_id
        WHERE agent.actor_id = $1 AND agent.review_status IN ($2, $3)
            AND capability.capability_key = $4 AND capability.request_status = $5
        FOR SHARE OF agent, capability",
    )
    .bind(agent_actor_id)
    .bind(AgentReviewStatus::Approved.as_database_value())
    .bind(AgentReviewStatus::Active.as_database_value())
    .bind(AgentCapability::Research.as_database_value())
    .bind(AgentCapabilityReviewStatus::Approved.as_database_value())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    authorized
        .map(|_| ())
        .ok_or(ResearchTaskError::AgentAssigneeUnavailable.into())
}

pub async fn assign_research_task_to_agent(
    pool: &PgPool,
    task_id: Uuid,
    manager_account_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let manager_actor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT identity.actor_id FROM human_actor_identities identity
        JOIN accounts account ON account.id = identity.account_id
        WHERE account.id = $1 AND account.account_status = 'active'
            AND account.email_verified_at IS NOT NULL FOR UPDATE OF account",
    )
    .bind(manager_account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let manager_actor_id = manager_actor_id.ok_or(ResearchTaskError::PermissionDenied)?;
    let task_project_and_status: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT project_id, task_status FROM research_tasks WHERE id = $1 FOR UPDATE",
    )
    .bind(task_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let (project_id, status) = task_project_and_status.ok_or(ResearchTaskError::TaskNotFound)?;
    let project_status: Option<String> =
        sqlx::query_scalar("SELECT project_status FROM research_projects WHERE id = $1 FOR UPDATE")
            .bind(project_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    if project_status.as_deref() != Some(ResearchProjectStatus::Active.as_database_value()) {
        return Err(ResearchTaskError::ProjectNotFound.into());
    }
    let can_manage: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM research_project_members
        WHERE project_id = $1 AND actor_id = $2 AND membership_status = 'active'
            AND member_role IN ('owner', 'supervisor', 'researcher'))",
    )
    .bind(project_id)
    .bind(manager_actor_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !can_manage {
        return Err(ResearchTaskError::PermissionDenied.into());
    }
    if status != ResearchTaskStatus::Queued.as_database_value() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    lock_approved_research_agent(&mut transaction, agent_actor_id).await?;
    sqlx::query(
        "UPDATE research_tasks SET assigned_actor_id = $2, task_status = $3,
            assignment_accepted_at = NULL, updated_at = now() WHERE id = $1",
    )
    .bind(task_id)
    .bind(agent_actor_id)
    .bind(ResearchTaskStatus::Assigned.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_assigned_research_tasks_for_agent(
    pool: &PgPool,
    agent_actor_id: Uuid,
    requested_limit: Option<i64>,
) -> Result<Vec<AgentResearchTaskSummary>, AppError> {
    let limit = requested_limit
        .unwrap_or(crate::constants::agents::DEFAULT_AGENT_RESEARCH_TASK_LIST_LIMIT)
        .clamp(1, MAX_RESEARCH_TASK_LIST_LIMIT);
    sqlx::query_as::<_, AgentResearchTaskSummary>(
        "SELECT task.id, task.title, task.task_status, task.updated_at
        FROM research_tasks task JOIN research_projects project ON project.id = task.project_id
        JOIN agent_actor_identities agent ON agent.actor_id = $1
        JOIN agent_capability_requests capability ON capability.agent_actor_id = agent.actor_id
        WHERE task.assigned_actor_id = $1 AND task.task_status IN ($2, $3)
            AND project.project_status = $4
            AND agent.review_status IN ($5, $6)
            AND capability.capability_key = $7 AND capability.request_status = $8
        ORDER BY task.updated_at DESC, task.id LIMIT $9",
    )
    .bind(agent_actor_id)
    .bind(ResearchTaskStatus::Assigned.as_database_value())
    .bind(ResearchTaskStatus::Running.as_database_value())
    .bind(ResearchProjectStatus::Active.as_database_value())
    .bind(AgentReviewStatus::Approved.as_database_value())
    .bind(AgentReviewStatus::Active.as_database_value())
    .bind(AgentCapability::Research.as_database_value())
    .bind(AgentCapabilityReviewStatus::Approved.as_database_value())
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn find_assigned_research_task_for_agent(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<AgentResearchTask, AppError> {
    sqlx::query_as::<_, AgentResearchTask>(
        "SELECT task.id, task.title, task.objective, task.task_scope, task.allowed_data_sources,
            task.permitted_tools, task.expected_deliverables, task.max_runtime_seconds,
            task.max_result_bytes, task.task_status, task.created_at, task.updated_at
        FROM research_tasks task JOIN research_projects project ON project.id = task.project_id
        JOIN agent_actor_identities agent ON agent.actor_id = $2
        JOIN agent_capability_requests capability ON capability.agent_actor_id = agent.actor_id
        WHERE task.id = $1 AND task.assigned_actor_id = $2
            AND task.task_status IN ($3, $4) AND project.project_status = $5
            AND agent.review_status IN ($6, $7)
            AND capability.capability_key = $8 AND capability.request_status = $9",
    )
    .bind(task_id)
    .bind(agent_actor_id)
    .bind(ResearchTaskStatus::Assigned.as_database_value())
    .bind(ResearchTaskStatus::Running.as_database_value())
    .bind(ResearchProjectStatus::Active.as_database_value())
    .bind(AgentReviewStatus::Approved.as_database_value())
    .bind(AgentReviewStatus::Active.as_database_value())
    .bind(AgentCapability::Research.as_database_value())
    .bind(AgentCapabilityReviewStatus::Approved.as_database_value())
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchTaskError::TaskNotFound.into())
}

pub async fn accept_assigned_research_task_for_agent(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    update_agent_task_status(
        pool,
        task_id,
        agent_actor_id,
        ResearchTaskStatus::Assigned,
        Some(true),
    )
    .await
}

pub async fn start_assigned_research_task_for_agent(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    update_agent_task_status(
        pool,
        task_id,
        agent_actor_id,
        ResearchTaskStatus::Assigned,
        Some(false),
    )
    .await
}

async fn update_agent_task_status(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
    expected_status: ResearchTaskStatus,
    accepting: Option<bool>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let row: Option<(String, Option<Uuid>, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(
            "SELECT task.task_status, task.assigned_actor_id, task.assignment_accepted_at
        FROM research_tasks task JOIN research_projects project ON project.id = task.project_id
        WHERE task.id = $1 AND project.project_status = 'active' FOR UPDATE OF task",
        )
        .bind(task_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    let (status, assigned_actor_id, accepted_at) = row.ok_or(ResearchTaskError::TaskNotFound)?;
    if assigned_actor_id != Some(agent_actor_id) {
        return Err(ResearchTaskError::AssignmentUnavailable.into());
    }
    lock_approved_research_agent(&mut transaction, agent_actor_id).await?;
    if status != expected_status.as_database_value() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    if accepting == Some(true) {
        if accepted_at.is_some() {
            return Err(ResearchTaskError::InvalidTransition.into());
        }
        sqlx::query("UPDATE research_tasks SET assignment_accepted_at = now(), updated_at = now() WHERE id = $1")
            .bind(task_id).execute(&mut *transaction).await.map_err(|_| AppError::Internal)?;
    } else if accepting == Some(false) {
        if accepted_at.is_none() {
            return Err(ResearchTaskError::InvalidTransition.into());
        }
        sqlx::query("UPDATE research_tasks SET task_status = $2, updated_at = now() WHERE id = $1")
            .bind(task_id)
            .bind(ResearchTaskStatus::Running.as_database_value())
            .execute(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    }
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn submit_research_task_result_for_agent(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
    result: &str,
    sources: &[AgentTaskSource],
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    super::super::agent_administration_repository::research_quota::acquire_agent_quota_lock(
        &mut transaction,
        agent_actor_id,
    )
    .await?;
    lock_approved_research_agent(&mut transaction, agent_actor_id).await?;
    enforce_agent_research_submission_quota(&mut transaction, agent_actor_id).await?;
    let task: Option<(String, i32, Option<Uuid>, Uuid)> = sqlx::query_as(
        "SELECT task.task_status, task.max_result_bytes, task.assigned_actor_id, task.project_id
        FROM research_tasks task JOIN research_projects project ON project.id = task.project_id
        WHERE task.id = $1 AND project.project_status = 'active' FOR UPDATE OF task",
    )
    .bind(task_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let (status, max_result_bytes, assignee, _) = task.ok_or(ResearchTaskError::TaskNotFound)?;
    if assignee != Some(agent_actor_id) {
        return Err(ResearchTaskError::AssignmentUnavailable.into());
    }
    if status != ResearchTaskStatus::Running.as_database_value() {
        return Err(ResearchTaskError::InvalidTransition.into());
    }
    if result.trim().is_empty() || result.len() > max_result_bytes as usize {
        return Err(ResearchTaskError::ResultInvalid.into());
    }
    if sources.is_empty() || sources.len() > MAX_RESEARCH_TASK_SOURCE_REFERENCES {
        return Err(ResearchTaskError::SourceReferencesInvalid.into());
    }
    let submission_number: i16 = sqlx::query_scalar(
        "SELECT (COALESCE(MAX(submission_number), 0) + 1)::smallint
        FROM research_task_submissions WHERE task_id = $1",
    )
    .bind(task_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if submission_number > MAX_RESEARCH_TASK_SUBMISSIONS {
        return Err(ResearchTaskError::SubmissionLimitReached.into());
    }
    let submission_id: Uuid = sqlx::query_scalar(
        "INSERT INTO research_task_submissions
            (task_id, submission_number, submitted_by_actor_id, result)
        VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(task_id)
    .bind(submission_number)
    .bind(agent_actor_id)
    .bind(result.trim())
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    for (position, source) in sources.iter().enumerate() {
        sqlx::query(
            "INSERT INTO research_task_submission_sources
                (submission_id, source_order, title, source_url, doi)
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(submission_id)
        .bind((position + 1) as i16)
        .bind(source.title.trim())
        .bind(source.url.trim())
        .bind(
            source
                .doi
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    sqlx::query("UPDATE research_tasks SET task_status = $2, updated_at = now() WHERE id = $1")
        .bind(task_id)
        .bind(ResearchTaskStatus::AwaitingReview.as_database_value())
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn enforce_agent_research_submission_quota(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    agent_actor_id: Uuid,
) -> Result<(), AppError> {
    let quota =
        super::super::agent_administration_repository::research_quota::find_agent_research_quota(
            transaction,
            agent_actor_id,
        )
        .await?;
    let submissions_in_window: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM research_task_submissions
        WHERE submitted_by_actor_id = $1
            AND submitted_at >= now() - make_interval(hours => $2::integer)",
    )
    .bind(agent_actor_id)
    .bind(crate::constants::agents::AGENT_RESEARCH_SUBMISSION_WINDOW_HOURS)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if submissions_in_window >= i64::from(quota) {
        return Err(AgentError::ResearchQuotaReached.into());
    }
    Ok(())
}
