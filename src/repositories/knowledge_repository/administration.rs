use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::{admin_auth, knowledge},
    enums::knowledge_error::KnowledgeError,
    enums::{admin_auth::AdministratorLifecycleStatus, knowledge::KnowledgeReviewDecision},
    errors::app_error::AppError,
    models::knowledge::{
        KnowledgeCitationRecord, KnowledgeEntrySummary, KnowledgeFindingProposalRecord,
        KnowledgeProposalDetailRecord,
    },
};

pub async fn find_submitted_knowledge_proposal(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Option<KnowledgeProposalDetailRecord>, AppError> {
    sqlx::query_as(
        "SELECT entry.id, entry.academic_subject_id, revision.title, revision.summary,
            revision.created_at
        FROM knowledge_entries entry
        JOIN knowledge_entry_revisions revision
            ON revision.entry_id = entry.id
            AND revision.revision_number = entry.latest_revision_number
        WHERE entry.id = $1 AND entry.entry_status = 'submitted'",
    )
    .bind(entry_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_submitted_proposal_citations(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Vec<KnowledgeCitationRecord>, AppError> {
    sqlx::query_as(
        "SELECT citation.title, citation.doi, citation.external_url, citation.authors,
            citation.publication_year, citation.license
        FROM knowledge_entries entry
        JOIN knowledge_entry_revisions revision
            ON revision.entry_id = entry.id
            AND revision.revision_number = entry.latest_revision_number
        JOIN knowledge_revision_citations link
            ON link.entry_id = revision.entry_id
            AND link.revision_number = revision.revision_number
        JOIN knowledge_citations citation ON citation.id = link.citation_id
        WHERE entry.id = $1 AND entry.entry_status = 'submitted'
        ORDER BY citation.title, citation.id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_submitted_proposal_research_findings(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Vec<KnowledgeFindingProposalRecord>, AppError> {
    sqlx::query_as(
        "SELECT link.finding_id, link.relation,
            CASE WHEN project.id IS NOT NULL AND revision.finding_id IS NOT NULL
                THEN revision.title ELSE NULL END AS public_title,
            (project.id IS NOT NULL AND revision.finding_id IS NOT NULL) AS currently_public
        FROM knowledge_entries entry
        JOIN knowledge_entry_revisions entry_revision
            ON entry_revision.entry_id = entry.id
            AND entry_revision.revision_number = entry.latest_revision_number
        JOIN knowledge_revision_research_findings link
            ON link.entry_id = entry_revision.entry_id
            AND link.revision_number = entry_revision.revision_number
        LEFT JOIN research_findings finding ON finding.id = link.finding_id
            AND finding.finding_status = 'published'
        LEFT JOIN research_projects project ON project.id = finding.project_id
            AND project.project_status = 'active' AND project.visibility = 'public'
        LEFT JOIN research_finding_revisions revision
            ON revision.finding_id = finding.id
            AND revision.revision_number = finding.published_revision_number
        WHERE entry.id = $1 AND entry.entry_status = 'submitted'
        ORDER BY link.finding_id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_submitted_knowledge_proposals(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<KnowledgeEntrySummary>, i64), AppError> {
    let total = sqlx::query_scalar(
        "SELECT count(*) FROM knowledge_entries WHERE entry_status = 'submitted'",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let items = sqlx::query_as(
        "SELECT entry.id, entry.academic_subject_id, revision.title, revision.summary,
            revision.created_at
        FROM knowledge_entries entry
        JOIN knowledge_entry_revisions revision
            ON revision.entry_id = entry.id AND revision.revision_number = entry.latest_revision_number
        WHERE entry.entry_status = 'submitted'
        ORDER BY entry.created_at, entry.id LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((items, total))
}

pub async fn review_knowledge_proposal(
    pool: &PgPool,
    entry_id: Uuid,
    administrator_id: Uuid,
    decision: KnowledgeReviewDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_authorization(&mut transaction).await?;
    ensure_knowledge_permission(
        &mut transaction,
        administrator_id,
        knowledge::ADMIN_KNOWLEDGE_REVIEW_PERMISSION,
    )
    .await?;
    let entry_status = lock_knowledge_entry(&mut transaction, entry_id).await?;
    if entry_status != "submitted" {
        return Err(KnowledgeError::ProposalNotAwaitingReview.into());
    }
    update_latest_revision_review(
        &mut transaction,
        entry_id,
        administrator_id,
        decision,
        reason,
    )
    .await?;
    sqlx::query("UPDATE knowledge_entries SET entry_status = $2, updated_at = now() WHERE id = $1")
        .bind(entry_id)
        .bind(decision.next_entry_status())
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    record_knowledge_administration_event(
        &mut transaction,
        entry_id,
        administrator_id,
        decision.audit_action(),
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn publish_approved_knowledge_entry(
    pool: &PgPool,
    entry_id: Uuid,
    administrator_id: Uuid,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    lock_administrator_authorization(&mut transaction).await?;
    ensure_knowledge_permission(
        &mut transaction,
        administrator_id,
        knowledge::ADMIN_KNOWLEDGE_PUBLISH_PERMISSION,
    )
    .await?;
    let status = lock_knowledge_entry(&mut transaction, entry_id).await?;
    if status != "approved" {
        return Err(KnowledgeError::EntryNotApproved.into());
    }
    let revision_number: i16 =
        sqlx::query_scalar("SELECT latest_revision_number FROM knowledge_entries WHERE id = $1")
            .bind(entry_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    let updated = sqlx::query(
        "UPDATE knowledge_entry_revisions SET revision_status = 'published'
        WHERE entry_id = $1 AND revision_number = $2 AND revision_status = 'approved'",
    )
    .bind(entry_id)
    .bind(revision_number)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if updated.rows_affected() != 1 {
        return Err(KnowledgeError::EntryNotPublishable.into());
    }
    sqlx::query(
        "UPDATE knowledge_entries SET entry_status = 'published',
            published_revision_number = latest_revision_number, updated_at = now()
        WHERE id = $1 AND entry_status = 'approved'",
    )
    .bind(entry_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    record_knowledge_administration_event(
        &mut transaction,
        entry_id,
        administrator_id,
        knowledge::KNOWLEDGE_ENTRY_PUBLISHED_AUDIT_ACTION,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn lock_knowledge_entry(
    transaction: &mut Transaction<'_, Postgres>,
    entry_id: Uuid,
) -> Result<String, AppError> {
    sqlx::query_scalar("SELECT entry_status FROM knowledge_entries WHERE id = $1 FOR UPDATE")
        .bind(entry_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(KnowledgeError::ProposalNotFound.into())
}

async fn lock_administrator_authorization(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), AppError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(admin_auth::ADMIN_ROLE_MANAGEMENT_ADVISORY_LOCK_ID)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn ensure_knowledge_permission(
    transaction: &mut Transaction<'_, Postgres>,
    administrator_id: Uuid,
    permission_key: &str,
) -> Result<(), AppError> {
    let permitted: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM administrator_roles assignment
            JOIN administrators administrator
                ON administrator.id = assignment.administrator_id
                AND administrator.status = $2
            JOIN roles role
                ON role.role_key = assignment.role_key AND role.is_active
            JOIN role_permissions permission ON permission.role_key = assignment.role_key
            WHERE assignment.administrator_id = $1 AND assignment.revoked_at IS NULL
                AND permission.permission_key = $3
        )",
    )
    .bind(administrator_id)
    .bind(AdministratorLifecycleStatus::Active.as_database_value())
    .bind(permission_key)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if permitted {
        Ok(())
    } else {
        Err(crate::enums::admin_auth_error::AdminAuthError::PermissionDenied.into())
    }
}

async fn update_latest_revision_review(
    transaction: &mut Transaction<'_, Postgres>,
    entry_id: Uuid,
    administrator_id: Uuid,
    decision: KnowledgeReviewDecision,
    reason: &str,
) -> Result<(), AppError> {
    let status = match decision {
        KnowledgeReviewDecision::Approve => "approved",
        KnowledgeReviewDecision::Reject => "rejected",
    };
    let result = sqlx::query(
        "UPDATE knowledge_entry_revisions revision
        SET revision_status = $2, reviewed_by_administrator_id = $3,
            review_reason = $4, reviewed_at = now()
        FROM knowledge_entries entry
        WHERE entry.id = $1 AND revision.entry_id = entry.id
            AND revision.revision_number = entry.latest_revision_number
            AND revision.revision_status = 'submitted'",
    )
    .bind(entry_id)
    .bind(status)
    .bind(administrator_id)
    .bind(reason)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(KnowledgeError::ProposalNotAwaitingReview.into())
    }
}

async fn record_knowledge_administration_event(
    transaction: &mut Transaction<'_, Postgres>,
    entry_id: Uuid,
    administrator_id: Uuid,
    action: &str,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id,
             result, reason, correlation_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(Uuid::new_v4())
    .bind(administrator_id)
    .bind(action)
    .bind(knowledge::KNOWLEDGE_AUDIT_TARGET_TYPE)
    .bind(entry_id)
    .bind(admin_auth::ADMIN_AUDIT_SUCCESS_RESULT)
    .bind(reason)
    .bind(correlation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}
