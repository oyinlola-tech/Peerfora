use sqlx::{PgPool, query_scalar};
use uuid::Uuid;

use crate::{
    constants::research::MAX_RESEARCH_FINDING_REVIEW_NOTES_LENGTH,
    enums::research::{ResearchFindingError, ResearchFindingReviewDecision},
    errors::app_error::AppError,
};

use super::super::access;

pub async fn submit_research_finding_for_review(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    let finding = access::lock_research_finding(&mut transaction, finding_id).await?;
    access::require_active_project(&mut transaction, finding.project_id).await?;
    access::require_project_writer(&mut *transaction, finding.project_id, actor_id).await?;
    let revision_status: Option<String> = query_scalar(
        "SELECT review_status FROM research_finding_revisions
        WHERE finding_id = $1 AND revision_number = $2 FOR UPDATE",
    )
    .bind(finding_id)
    .bind(finding.latest_revision_number)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !matches!(
        (finding.finding_status.as_str(), revision_status.as_deref()),
        ("draft" | "correction_draft", Some("draft"))
    ) {
        return Err(ResearchFindingError::InvalidTransition.into());
    }
    let status = if finding.published_revision_number.is_some() {
        "correction_in_review"
    } else {
        "in_review"
    };
    sqlx::query(
        "UPDATE research_finding_revisions SET review_status = 'in_review'
        WHERE finding_id = $1 AND revision_number = $2",
    )
    .bind(finding_id)
    .bind(finding.latest_revision_number)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE research_findings SET finding_status = $2, updated_at = now()
        WHERE id = $1",
    )
    .bind(finding_id)
    .bind(status)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn review_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
    decision: ResearchFindingReviewDecision,
    notes: Option<String>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let reviewer_actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    let finding = access::lock_research_finding(&mut transaction, finding_id).await?;
    access::require_active_project(&mut transaction, finding.project_id).await?;
    access::require_project_reviewer(&mut *transaction, finding.project_id, reviewer_actor_id)
        .await?;
    let author_actor_id: Option<Uuid> = query_scalar(
        "SELECT authored_by_actor_id FROM research_finding_revisions
        WHERE finding_id = $1 AND revision_number = $2 AND review_status = 'in_review'
        FOR UPDATE",
    )
    .bind(finding_id)
    .bind(finding.latest_revision_number)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let author_actor_id = author_actor_id.ok_or(ResearchFindingError::ReviewUnavailable)?;
    if author_actor_id == reviewer_actor_id {
        return Err(ResearchFindingError::SelfReviewForbidden.into());
    }
    if notes
        .as_deref()
        .is_some_and(|value| value.chars().count() > MAX_RESEARCH_FINDING_REVIEW_NOTES_LENGTH)
    {
        return Err(ResearchFindingError::ReviewNotesInvalid.into());
    }

    sqlx::query(
        "UPDATE research_finding_revisions SET review_status = $3,
            reviewed_by_actor_id = $4, reviewer_notes = $5, reviewed_at = now()
        WHERE finding_id = $1 AND revision_number = $2",
    )
    .bind(finding_id)
    .bind(finding.latest_revision_number)
    .bind(decision.as_database_value())
    .bind(reviewer_actor_id)
    .bind(notes)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    let status = match (decision, finding.published_revision_number.is_some()) {
        (ResearchFindingReviewDecision::Approve, false) => "approved",
        (ResearchFindingReviewDecision::Approve, true) => "correction_approved",
        (ResearchFindingReviewDecision::Reject, false) => "draft",
        (ResearchFindingReviewDecision::Reject, true) => "correction_draft",
    };
    sqlx::query(
        "UPDATE research_findings SET finding_status = $2, updated_at = now()
        WHERE id = $1",
    )
    .bind(finding_id)
    .bind(status)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn publish_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let publisher_actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    let finding = access::lock_research_finding(&mut transaction, finding_id).await?;
    access::require_active_project(&mut transaction, finding.project_id).await?;
    let role =
        access::require_project_member(&mut *transaction, finding.project_id, publisher_actor_id)
            .await?;
    if !matches!(role.as_str(), "owner" | "supervisor") {
        return Err(ResearchFindingError::PermissionDenied.into());
    }
    let approved: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM research_finding_revisions WHERE finding_id = $1
                AND revision_number = $2 AND review_status = 'approved'
        )",
    )
    .bind(finding_id)
    .bind(finding.latest_revision_number)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !approved
        || !matches!(
            finding.finding_status.as_str(),
            "approved" | "correction_approved"
        )
    {
        return Err(ResearchFindingError::InvalidTransition.into());
    }
    supersede_published_revision(&mut transaction, &finding).await?;
    sqlx::query(
        "UPDATE research_finding_revisions SET review_status = 'published'
        WHERE finding_id = $1 AND revision_number = $2",
    )
    .bind(finding_id)
    .bind(finding.latest_revision_number)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    sqlx::query(
        "UPDATE research_findings SET finding_status = 'published',
            published_revision_number = latest_revision_number, updated_at = now()
        WHERE id = $1",
    )
    .bind(finding_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn supersede_published_revision(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    finding: &access::LockedResearchFinding,
) -> Result<(), AppError> {
    if let Some(revision_number) = finding.published_revision_number {
        sqlx::query(
            "UPDATE research_finding_revisions SET review_status = 'superseded'
            WHERE finding_id = $1 AND revision_number = $2",
        )
        .bind(finding.id)
        .bind(revision_number)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}

pub async fn retract_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id = access::find_active_human_actor(&mut transaction, account_id).await?;
    let finding = access::lock_research_finding(&mut transaction, finding_id).await?;
    access::require_active_project(&mut transaction, finding.project_id).await?;
    let role =
        access::require_project_member(&mut *transaction, finding.project_id, actor_id).await?;
    let may_retract =
        finding.created_by_actor_id == actor_id || matches!(role.as_str(), "owner" | "supervisor");
    if !may_retract {
        return Err(ResearchFindingError::PermissionDenied.into());
    }
    if finding.published_revision_number.is_none()
        || matches!(
            finding.finding_status.as_str(),
            "retracted" | "draft" | "in_review" | "approved"
        )
    {
        return Err(ResearchFindingError::InvalidTransition.into());
    }
    sqlx::query(
        "UPDATE research_findings SET finding_status = 'retracted',
            retracted_at = now(), updated_at = now() WHERE id = $1",
    )
    .bind(finding_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}
