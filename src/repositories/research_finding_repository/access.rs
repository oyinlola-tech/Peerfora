use sqlx::{PgConnection, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::research::ResearchFindingError,
    errors::app_error::AppError,
};

#[derive(sqlx::FromRow)]
pub struct LockedResearchFinding {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_by_actor_id: Uuid,
    pub finding_status: String,
    pub latest_revision_number: i16,
    pub published_revision_number: Option<i16>,
}

pub async fn find_active_human_actor(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<Uuid, AppError> {
    let actor_id = sqlx::query_scalar(
        "SELECT identity.actor_id FROM human_actor_identities identity
        JOIN accounts ON accounts.id = identity.account_id
        WHERE accounts.id = $1 AND accounts.account_status = 'active'
            AND accounts.email_verified_at IS NOT NULL FOR SHARE OF accounts",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchFindingError::PermissionDenied)?;
    Ok(actor_id)
}

pub async fn require_active_project(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<(), AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM research_projects
            WHERE id = $1 AND project_status = 'active' FOR UPDATE)",
    )
    .bind(project_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if exists {
        Ok(())
    } else {
        Err(ResearchFindingError::ProjectNotFound.into())
    }
}

pub async fn require_project_member(
    connection: &mut PgConnection,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<String, AppError> {
    sqlx::query_scalar(
        "SELECT member.member_role FROM research_project_members member
        WHERE member.project_id = $1 AND member.actor_id = $2
            AND member.membership_status = 'active'",
    )
    .bind(project_id)
    .bind(actor_id)
    .fetch_optional(connection)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or_else(|| ResearchFindingError::ProjectNotFound.into())
}

pub async fn require_project_writer(
    connection: &mut PgConnection,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let role = require_project_member(connection, project_id, actor_id).await?;
    if matches!(role.as_str(), "owner" | "supervisor" | "researcher" | "contributor") {
        Ok(())
    } else {
        Err(ResearchFindingError::PermissionDenied.into())
    }
}

pub async fn require_project_reviewer(
    connection: &mut PgConnection,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let role = require_project_member(connection, project_id, actor_id).await?;
    if matches!(role.as_str(), "owner" | "supervisor" | "reviewer") {
        Ok(())
    } else {
        Err(ResearchFindingError::PermissionDenied.into())
    }
}

pub async fn lock_research_finding(
    transaction: &mut Transaction<'_, Postgres>,
    finding_id: Uuid,
) -> Result<LockedResearchFinding, AppError> {
    sqlx::query_as::<_, LockedResearchFinding>(
        "SELECT id, project_id, created_by_actor_id, finding_status,
            latest_revision_number, published_revision_number
        FROM research_findings WHERE id = $1 FOR UPDATE",
    )
    .bind(finding_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or_else(|| ResearchFindingError::FindingNotFound.into())
}

pub fn map_duplicate_doi(error: sqlx::Error) -> AppError {
    if error.as_database_error().and_then(|error| error.code()).as_deref() == Some("23505") {
        ResearchFindingError::DuplicateDoi.into()
    } else {
        AppError::Internal
    }
}
