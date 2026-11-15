use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::research::ResearchProjectError, errors::app_error::AppError,
    models::research_project::ResearchProject,
};

pub struct ProjectUpdate<'a> {
    pub title: &'a str,
    pub abstract_text: Option<&'a str>,
    pub description: &'a str,
    pub objectives: &'a [String],
    pub research_questions: &'a [String],
    pub subject_ids: &'a [Uuid],
}

pub async fn update_research_project(
    pool: &PgPool,
    project_id: Uuid,
    owner_account_id: Uuid,
    details: ProjectUpdate<'_>,
) -> Result<ResearchProject, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let owner_actor_id = find_active_account_actor(&mut transaction, owner_account_id).await?;
    ensure_project_owner(&mut transaction, project_id, owner_actor_id).await?;
    ensure_subjects_are_active(&mut transaction, details.subject_ids).await?;
    sqlx::query(
        "UPDATE research_projects SET title = $2, abstract = $3, description = $4,
            objectives = $5, research_questions = $6, updated_at = now()
        WHERE id = $1 AND project_status = 'active'",
    )
    .bind(project_id)
    .bind(details.title)
    .bind(details.abstract_text)
    .bind(details.description)
    .bind(details.objectives)
    .bind(details.research_questions)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    sqlx::query("DELETE FROM research_project_subjects WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "INSERT INTO research_project_subjects (project_id, subject_id)
        SELECT $1, subject_id FROM unnest($2::uuid[]) AS subject_id",
    )
    .bind(project_id)
    .bind(details.subject_ids)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    super::queries::find_research_project_visible_to_actor(pool, project_id, Some(owner_actor_id))
        .await?
        .ok_or(AppError::Internal)
}

pub async fn archive_research_project(
    pool: &PgPool,
    project_id: Uuid,
    owner_account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let owner_actor_id = find_active_account_actor(&mut transaction, owner_account_id).await?;
    ensure_project_owner(&mut transaction, project_id, owner_actor_id).await?;
    let archived = sqlx::query(
        "UPDATE research_projects SET project_status = 'archived',
            archived_at = now(), updated_at = now()
        WHERE id = $1 AND project_status = 'active'",
    )
    .bind(project_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .rows_affected()
        == 1;
    if !archived {
        return Err(ResearchProjectError::ProjectUnavailable.into());
    }
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn find_active_account_actor(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT identity.actor_id FROM human_actor_identities identity
        JOIN accounts account ON account.id = identity.account_id
        WHERE account.id = $1 AND account.account_status = 'active'
        FOR UPDATE OF account",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchProjectError::ProjectNotFound.into())
}

async fn ensure_project_owner(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let is_owner: bool = sqlx::query_scalar(
        "SELECT owner_actor_id = $2 AND project_status = 'active'
        FROM research_projects WHERE id = $1 FOR UPDATE",
    )
    .bind(project_id)
    .bind(actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .unwrap_or(false);
    if is_owner {
        Ok(())
    } else {
        Err(ResearchProjectError::PermissionDenied.into())
    }
}

async fn ensure_subjects_are_active(
    transaction: &mut Transaction<'_, Postgres>,
    subject_ids: &[Uuid],
) -> Result<(), AppError> {
    if subject_ids.is_empty() {
        return Ok(());
    }
    let active_subject_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM academic_subjects
        WHERE id = ANY($1) AND is_active = TRUE",
    )
    .bind(subject_ids)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if active_subject_count == subject_ids.len() as i64 {
        Ok(())
    } else {
        Err(ResearchProjectError::SubjectUnavailable.into())
    }
}
