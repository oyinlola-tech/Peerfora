use sqlx::{PgPool, Postgres, Transaction, query_as, query_scalar};
use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_RESEARCH_PROJECT_READ_AUDIT_ACTION, ADMIN_RESEARCH_PROJECT_RESTORED_AUDIT_ACTION,
        ADMIN_RESEARCH_PROJECT_SUSPENDED_AUDIT_ACTION,
    },
    enums::{admin_auth_error::AdminAuthError, research::ResearchProjectStatus},
    errors::app_error::AppError,
    models::admin_research::{
        AdministrativeResearchProjectFilters, AdministrativeResearchProjectRecord,
    },
    repositories::admin_auth_repository::{
        AdministrativeAuditEvent, insert_administrative_audit_event,
    },
};

pub async fn search_research_projects_for_administration(
    pool: &PgPool,
    filters: AdministrativeResearchProjectFilters<'_>,
    administrator_id: Uuid,
    limit: i64,
    offset: i64,
    correlation_id: Option<&str>,
) -> Result<(Vec<AdministrativeResearchProjectRecord>, i64), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let total_count = count_research_projects(&mut transaction, &filters).await?;
    let records = find_research_project_page(&mut transaction, &filters, limit, offset).await?;
    record_research_project_metadata_reads(
        &mut transaction,
        &records,
        administrator_id,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok((records, total_count))
}

pub async fn find_research_project_for_administration(
    pool: &PgPool,
    project_id: Uuid,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<AdministrativeResearchProjectRecord, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let record = find_research_project_in_transaction(&mut transaction, project_id)
        .await?
        .ok_or(AdminAuthError::ResearchProjectNotFound)?;
    record_research_project_metadata_read(
        &mut transaction,
        project_id,
        administrator_id,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(record)
}

pub async fn change_research_project_status(
    pool: &PgPool,
    project_id: Uuid,
    administrator_id: Uuid,
    expected_status: ResearchProjectStatus,
    new_status: ResearchProjectStatus,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let current_status: Option<String> =
        query_scalar("SELECT project_status FROM research_projects WHERE id = $1 FOR UPDATE")
            .bind(project_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    let current_status = current_status.ok_or(AdminAuthError::ResearchProjectNotFound)?;
    if current_status != expected_status.as_database_value() {
        return Err(AdminAuthError::ResearchProjectStateConflict.into());
    }
    sqlx::query(
        "UPDATE research_projects SET project_status = $2, updated_at = now() WHERE id = $1",
    )
    .bind(project_id)
    .bind(new_status.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let action = match new_status {
        ResearchProjectStatus::Suspended => ADMIN_RESEARCH_PROJECT_SUSPENDED_AUDIT_ACTION,
        ResearchProjectStatus::Active => ADMIN_RESEARCH_PROJECT_RESTORED_AUDIT_ACTION,
        ResearchProjectStatus::Archived => return Err(AppError::Internal),
    };
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action,
            target_type: "research_project",
            target_id: project_id,
            result: "success",
            reason: Some(reason_code),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn count_research_projects(
    transaction: &mut Transaction<'_, Postgres>,
    filters: &AdministrativeResearchProjectFilters<'_>,
) -> Result<i64, AppError> {
    query_scalar(
        "SELECT count(*) FROM research_projects project
        JOIN human_actor_identities owner_identity
            ON owner_identity.actor_id = project.owner_actor_id
        JOIN accounts owner ON owner.id = owner_identity.account_id
        WHERE ($1::text IS NULL OR lower(project.title) LIKE $1 || '%' ESCAPE E'\\\\'
            OR lower(owner.username) LIKE $1 || '%' ESCAPE E'\\\\')
            AND ($2::text IS NULL OR project.project_status = $2)",
    )
    .bind(filters.search_prefix)
    .bind(
        filters
            .project_status
            .map(ResearchProjectStatus::as_database_value),
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}

async fn find_research_project_page(
    transaction: &mut Transaction<'_, Postgres>,
    filters: &AdministrativeResearchProjectFilters<'_>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AdministrativeResearchProjectRecord>, AppError> {
    query_as::<_, AdministrativeResearchProjectRecord>(
        "SELECT project.id, owner.username AS owner_username, project.title,
            project.visibility, project.project_status,
            (SELECT count(*) FROM research_project_members member
                WHERE member.project_id = project.id AND member.membership_status = 'active')
                AS active_member_count,
            (SELECT count(*) FROM research_findings finding
                WHERE finding.project_id = project.id AND finding.finding_status <> 'retracted')
                AS finding_count,
            (SELECT count(*) FROM research_tasks task WHERE task.project_id = project.id)
                AS task_count,
            project.created_at, project.updated_at
        FROM research_projects project
        JOIN human_actor_identities owner_identity
            ON owner_identity.actor_id = project.owner_actor_id
        JOIN accounts owner ON owner.id = owner_identity.account_id
        WHERE ($1::text IS NULL OR lower(project.title) LIKE $1 || '%' ESCAPE E'\\\\'
            OR lower(owner.username) LIKE $1 || '%' ESCAPE E'\\\\')
            AND ($2::text IS NULL OR project.project_status = $2)
        ORDER BY project.updated_at DESC, project.id LIMIT $3 OFFSET $4",
    )
    .bind(filters.search_prefix)
    .bind(
        filters
            .project_status
            .map(ResearchProjectStatus::as_database_value),
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}

async fn find_research_project_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<Option<AdministrativeResearchProjectRecord>, AppError> {
    query_as::<_, AdministrativeResearchProjectRecord>(
        "SELECT project.id, owner.username AS owner_username, project.title,
            project.visibility, project.project_status,
            (SELECT count(*) FROM research_project_members member
                WHERE member.project_id = project.id AND member.membership_status = 'active')
                AS active_member_count,
            (SELECT count(*) FROM research_findings finding
                WHERE finding.project_id = project.id AND finding.finding_status <> 'retracted')
                AS finding_count,
            (SELECT count(*) FROM research_tasks task WHERE task.project_id = project.id)
                AS task_count,
            project.created_at, project.updated_at
        FROM research_projects project
        JOIN human_actor_identities owner_identity
            ON owner_identity.actor_id = project.owner_actor_id
        JOIN accounts owner ON owner.id = owner_identity.account_id
        WHERE project.id = $1",
    )
    .bind(project_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)
}

async fn record_research_project_metadata_reads(
    transaction: &mut Transaction<'_, Postgres>,
    records: &[AdministrativeResearchProjectRecord],
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    for project in records {
        record_research_project_metadata_read(
            transaction,
            project.id,
            administrator_id,
            correlation_id,
        )
        .await?;
    }
    Ok(())
}

async fn record_research_project_metadata_read(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    insert_administrative_audit_event(
        transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action: ADMIN_RESEARCH_PROJECT_READ_AUDIT_ACTION,
            target_type: "research_project",
            target_id: project_id,
            result: "success",
            reason: None,
            correlation_id,
        },
    )
    .await
}
