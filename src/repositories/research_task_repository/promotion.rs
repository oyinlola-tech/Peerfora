use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use super::access;

use crate::{
    constants::research::RESEARCH_AGENT_SOURCE_PROVENANCE,
    enums::research::{
        ResearchFindingRevisionStatus, ResearchFindingStatus, ResearchSourceType,
        ResearchTaskError, ResearchTaskReviewDecision, ResearchTaskStatus,
    },
    errors::app_error::AppError,
    validations::research_finding_validation::ValidatedResearchFinding,
};

pub async fn promote_approved_agent_submission_to_finding(
    pool: &sqlx::PgPool,
    task_id: Uuid,
    submission_number: i16,
    promoter_account_id: Uuid,
    finding_id: Uuid,
    finding: &ValidatedResearchFinding,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let promoter_actor_id =
        access::find_active_human_actor_for_account(&mut transaction, promoter_account_id).await?;
    let task = access::lock_research_task(&mut transaction, task_id).await?;
    access::ensure_project_task_manager(&mut transaction, task.project_id, promoter_actor_id)
        .await?;
    if task.task_status != ResearchTaskStatus::Completed.as_database_value() {
        return Err(ResearchTaskError::SubmissionUnavailable.into());
    }
    let submission: Option<(Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT id, submitted_by_actor_id, result FROM research_task_submissions
        WHERE task_id = $1 AND submission_number = $2 AND review_status = $3
            AND promoted_finding_id IS NULL
            AND submission_number = (SELECT MAX(submission_number)
                FROM research_task_submissions WHERE task_id = $1)
        FOR UPDATE",
    )
    .bind(task_id)
    .bind(submission_number)
    .bind(ResearchTaskReviewDecision::Approved.as_database_value())
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let (submission_id, agent_actor_id, submission_result) =
        submission.ok_or(ResearchTaskError::SubmissionUnavailable)?;
    if finding.statement != submission_result {
        return Err(ResearchTaskError::AgentResultMustBePreserved.into());
    }
    let citations = load_submission_source_ids(&mut transaction, submission_id).await?;
    if citations.is_empty() {
        return Err(ResearchTaskError::SourceReferencesInvalid.into());
    }

    sqlx::query(
        "INSERT INTO research_findings (
            id, project_id, created_by_actor_id, finding_status, latest_revision_number
        ) VALUES ($1, $2, $3, $4, 1)",
    )
    .bind(finding_id)
    .bind(task.project_id)
    .bind(promoter_actor_id)
    .bind(ResearchFindingStatus::InReview.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "INSERT INTO research_finding_revisions (
            finding_id, project_id, revision_number, title, statement, details,
            authored_by_actor_id, review_status
        ) VALUES ($1, $2, 1, $3, $4, $5, $6, $7)",
    )
    .bind(finding_id)
    .bind(task.project_id)
    .bind(&finding.title)
    .bind(&finding.statement)
    .bind(&finding.details)
    .bind(agent_actor_id)
    .bind(ResearchFindingRevisionStatus::InReview.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    for source_id in citations {
        sqlx::query(
            "INSERT INTO research_finding_revision_sources (
                finding_id, project_id, revision_number, source_id, added_by_actor_id
            ) VALUES ($1, $2, 1, $3, $4)",
        )
        .bind(finding_id)
        .bind(task.project_id)
        .bind(source_id)
        .bind(agent_actor_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    sqlx::query("UPDATE research_task_submissions SET promoted_finding_id = $2 WHERE id = $1")
        .bind(submission_id)
        .bind(finding_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn load_submission_source_ids(
    transaction: &mut Transaction<'_, Postgres>,
    submission_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let sources = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT title, source_url, doi FROM research_task_submission_sources
        WHERE submission_id = $1 ORDER BY source_order",
    )
    .bind(submission_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let mut source_ids = Vec::with_capacity(sources.len());
    for (title, source_url, doi) in sources {
        let source_id = insert_or_find_agent_source(
            transaction,
            submission_id,
            &title,
            &source_url,
            doi.as_deref(),
        )
        .await?;
        if !source_ids.contains(&source_id) {
            source_ids.push(source_id);
        }
    }
    Ok(source_ids)
}

async fn insert_or_find_agent_source(
    transaction: &mut Transaction<'_, Postgres>,
    submission_id: Uuid,
    title: &str,
    source_url: &str,
    doi: Option<&str>,
) -> Result<Uuid, AppError> {
    let source_owner: Uuid = sqlx::query_scalar(
        "SELECT submission.submitted_by_actor_id FROM research_task_submissions submission
        WHERE submission.id = $1",
    )
    .bind(submission_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let inserted_source_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO research_sources (
            project_id, source_type, title, doi, source_url, provenance, created_by_actor_id
        ) SELECT task.project_id, $7, $2, $4, $3, $5, $6
        FROM research_tasks task JOIN research_task_submissions submission
            ON submission.task_id = task.id
        WHERE submission.id = $1
        ON CONFLICT (project_id, lower(doi)) WHERE doi IS NOT NULL DO NOTHING
        RETURNING id",
    )
    .bind(submission_id)
    .bind(title)
    .bind(source_url)
    .bind(doi)
    .bind(RESEARCH_AGENT_SOURCE_PROVENANCE)
    .bind(source_owner)
    .bind(ResearchSourceType::ExternalUrl.as_database_value())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if let Some(source_id) = inserted_source_id {
        return Ok(source_id);
    }
    let Some(doi) = doi else {
        return Err(AppError::Internal);
    };
    let existing_source_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT source.id FROM research_sources source
        JOIN research_tasks task ON task.project_id = source.project_id
        JOIN research_task_submissions submission ON submission.task_id = task.id
        WHERE submission.id = $1 AND lower(source.doi) = lower($2) AND $2 IS NOT NULL
        ORDER BY source.id LIMIT 1",
    )
    .bind(submission_id)
    .bind(doi)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    existing_source_id.ok_or(AppError::Internal)
}
