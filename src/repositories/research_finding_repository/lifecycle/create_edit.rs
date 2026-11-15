use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::research::{ResearchFindingError, ResearchFindingStatus},
    errors::app_error::AppError,
    validations::research_finding_validation::ValidatedResearchFinding,
};

use super::super::access;
use super::support::{ensure_citations_belong_to_project, insert_finding_revision};

pub async fn create_research_finding(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    finding_id: Uuid,
    details: ValidatedResearchFinding,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let author_actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    access::require_active_project(&mut transaction, project_id).await?;
    access::require_project_writer(&mut *transaction, project_id, author_actor_id).await?;
    ensure_citations_belong_to_project(&mut transaction, project_id, &details).await?;

    sqlx::query(
        "INSERT INTO research_findings (
            id, project_id, created_by_actor_id, finding_status, latest_revision_number
        ) VALUES ($1, $2, $3, $4, 1)",
    )
    .bind(finding_id)
    .bind(project_id)
    .bind(author_actor_id)
    .bind(ResearchFindingStatus::Draft.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    insert_finding_revision(
        &mut transaction,
        finding_id,
        project_id,
        1,
        author_actor_id,
        &details,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn update_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
    details: ValidatedResearchFinding,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let author_actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    let finding = access::lock_research_finding(&mut transaction, finding_id).await?;
    access::require_active_project(&mut transaction, finding.project_id).await?;
    access::require_project_writer(&mut *transaction, finding.project_id, author_actor_id).await?;
    if !matches!(
        ResearchFindingStatus::from_database_value(&finding.finding_status),
        Some(
            ResearchFindingStatus::Draft
                | ResearchFindingStatus::Approved
                | ResearchFindingStatus::Published
                | ResearchFindingStatus::CorrectionDraft
                | ResearchFindingStatus::CorrectionApproved
        )
    ) {
        return Err(ResearchFindingError::InvalidTransition.into());
    }
    if finding.latest_revision_number >= crate::constants::research::MAX_RESEARCH_FINDING_REVISIONS
    {
        return Err(ResearchFindingError::RevisionLimitReached.into());
    }
    ensure_citations_belong_to_project(&mut transaction, finding.project_id, &details).await?;

    let revision_number = finding.latest_revision_number + 1;
    let status = if finding.published_revision_number.is_some() {
        ResearchFindingStatus::CorrectionDraft
    } else {
        ResearchFindingStatus::Draft
    };
    update_latest_revision_pointer(&mut transaction, finding.id, revision_number, status).await?;
    insert_finding_revision(
        &mut transaction,
        finding.id,
        finding.project_id,
        revision_number,
        author_actor_id,
        &details,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn update_latest_revision_pointer(
    transaction: &mut Transaction<'_, Postgres>,
    finding_id: Uuid,
    revision_number: i16,
    status: ResearchFindingStatus,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE research_findings SET latest_revision_number = $2,
            finding_status = $3, updated_at = now() WHERE id = $1",
    )
    .bind(finding_id)
    .bind(revision_number)
    .bind(status.as_database_value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
