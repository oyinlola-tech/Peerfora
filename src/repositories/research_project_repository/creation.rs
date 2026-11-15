use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::research::ResearchProjectError, errors::app_error::AppError,
    models::research_project::ResearchProject,
};

pub struct NewResearchProject<'a> {
    pub title: &'a str,
    pub abstract_text: Option<&'a str>,
    pub description: &'a str,
    pub objectives: &'a [String],
    pub research_questions: &'a [String],
    pub subject_ids: &'a [Uuid],
    pub visibility: &'a str,
}

pub async fn create_research_project(
    pool: &PgPool,
    owner_account_id: Uuid,
    project_id: Uuid,
    details: NewResearchProject<'_>,
) -> Result<ResearchProject, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let owner_actor_id = find_active_human_actor(&mut transaction, owner_account_id).await?;
    ensure_subjects_are_active(&mut transaction, details.subject_ids).await?;

    sqlx::query(
        "INSERT INTO research_projects (
            id, owner_actor_id, title, abstract, description, objectives,
            research_questions, visibility
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(project_id)
    .bind(owner_actor_id)
    .bind(details.title)
    .bind(details.abstract_text)
    .bind(details.description)
    .bind(details.objectives)
    .bind(details.research_questions)
    .bind(details.visibility)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    sqlx::query(
        "INSERT INTO research_project_members (
            project_id, actor_id, member_role, membership_status, joined_at
        ) VALUES ($1, $2, 'owner', 'active', now())",
    )
    .bind(project_id)
    .bind(owner_actor_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    insert_project_subjects(&mut transaction, project_id, details.subject_ids).await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;

    super::queries::find_research_project_visible_to_actor(pool, project_id, Some(owner_actor_id))
        .await?
        .ok_or(AppError::Internal)
}

async fn find_active_human_actor(
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
    .ok_or(ResearchProjectError::AccountUnavailable.into())
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

async fn insert_project_subjects(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    subject_ids: &[Uuid],
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO research_project_subjects (project_id, subject_id)
        SELECT $1, subject_id FROM unnest($2::uuid[]) AS subject_id",
    )
    .bind(project_id)
    .bind(subject_ids)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
