use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::moderation::{
        MODERATION_REPORT_ASSIGNED_AUDIT_ACTION, MODERATION_REPORT_ASSIGNMENT_REASON_PREFIX,
        MODERATION_REPORT_AUDIT_TARGET_TYPE, MODERATION_REPORT_DISMISSED_AUDIT_ACTION,
        MODERATION_REPORT_RESOLVED_AUDIT_ACTION,
    },
    enums::moderation::{
        ModerationError, ModerationReportCategory, ModerationReportDecision, ModerationReportStatus,
    },
    errors::app_error::AppError,
    models::{moderation::ModerationReportPage, moderation::ModerationReportTarget},
};

pub async fn create_report(
    pool: &PgPool,
    reporter_account_id: Uuid,
    target: ModerationReportTarget,
    category: ModerationReportCategory,
    description: &str,
) -> Result<(Uuid, DateTime<Utc>), AppError> {
    let category = category.as_database_value();
    let inserted = match target {
        ModerationReportTarget::Account(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_account_id, category, description)
                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter
                JOIN accounts target ON target.id = $2 AND target.account_status = 'active'
                WHERE reporter.id = $1 AND reporter.account_status = 'active'
                    AND reporter.email_verified_at IS NOT NULL AND reporter.id <> target.id
                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::Community(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_community_id, category, description)
                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter
                JOIN communities target ON target.id = $2 AND target.lifecycle_status = 'active'
                WHERE reporter.id = $1 AND reporter.account_status = 'active'
                    AND reporter.email_verified_at IS NOT NULL
                    AND (target.visibility = 'public' OR EXISTS (
                        SELECT 1 FROM community_members membership
                        WHERE membership.community_id = target.id AND membership.account_id = reporter.id
                            AND membership.membership_status = 'active'
                    ))
                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::Post(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_post_id, category, description)
                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter
                JOIN community_posts target ON target.id = $2 AND target.lifecycle_status IN ('active', 'locked')
                JOIN communities community ON community.id = target.community_id AND community.lifecycle_status = 'active'
                WHERE reporter.id = $1 AND reporter.account_status = 'active'
                    AND reporter.email_verified_at IS NOT NULL
                    AND (community.visibility = 'public' OR EXISTS (
                        SELECT 1 FROM community_members membership
                        WHERE membership.community_id = community.id AND membership.account_id = reporter.id
                            AND membership.membership_status = 'active'
                    ))
                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::Comment(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_comment_id, category, description)
                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter
                JOIN community_post_comments target ON target.id = $2 AND target.lifecycle_status = 'active'
                JOIN community_posts post ON post.id = target.post_id AND post.lifecycle_status IN ('active', 'locked')
                JOIN communities community ON community.id = post.community_id AND community.lifecycle_status = 'active'
                WHERE reporter.id = $1 AND reporter.account_status = 'active'
                    AND reporter.email_verified_at IS NOT NULL
                    AND (community.visibility = 'public' OR EXISTS (
                        SELECT 1 FROM community_members membership
                        WHERE membership.community_id = community.id AND membership.account_id = reporter.id
                            AND membership.membership_status = 'active'
                    ))
                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::ResearchProject(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_research_project_id, category, description)
                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter
                JOIN research_projects target ON target.id = $2 AND target.project_status = 'active'
                JOIN human_actor_identities human ON human.account_id = reporter.id
                WHERE reporter.id = $1 AND reporter.account_status = 'active'
                    AND reporter.email_verified_at IS NOT NULL
                    AND (target.visibility = 'public' OR EXISTS (
                        SELECT 1 FROM research_project_members member
                        WHERE member.project_id = target.id AND member.actor_id = human.actor_id
                            AND member.membership_status = 'active'
                    ))
                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::ResearchFinding(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_research_finding_id, category, description)
                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter
                JOIN research_findings target ON target.id = $2 AND target.finding_status IN ('published', 'retracted')
                JOIN research_projects project ON project.id = target.project_id AND project.project_status = 'active'
                JOIN human_actor_identities human ON human.account_id = reporter.id
                WHERE reporter.id = $1 AND reporter.account_status = 'active'
                    AND reporter.email_verified_at IS NOT NULL
                    AND (project.visibility = 'public' OR EXISTS (
                        SELECT 1 FROM research_project_members member
                        WHERE member.project_id = project.id AND member.actor_id = human.actor_id
                            AND member.membership_status = 'active'
                    ))
                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::Conversation(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_conversation_id, category, description)\n                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter\n                JOIN conversations target ON target.id = $2 AND target.lifecycle_status IN ('active', 'archived')\n                WHERE reporter.id = $1 AND reporter.account_status = 'active'\n                    AND reporter.email_verified_at IS NOT NULL\n                    AND EXISTS (\n                        SELECT 1 FROM conversation_members membership\n                        WHERE membership.conversation_id = target.id AND membership.account_id = reporter.id\n                            AND membership.membership_status = 'active'\n                    )\n                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::Message(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_message_id, category, description)\n                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter\n                JOIN messages target ON target.id = $2\n                JOIN conversations conversation ON conversation.id = target.conversation_id\n                    AND conversation.lifecycle_status IN ('active', 'archived')\n                WHERE reporter.id = $1 AND reporter.account_status = 'active'\n                    AND reporter.email_verified_at IS NOT NULL\n                    AND EXISTS (\n                        SELECT 1 FROM conversation_members membership\n                        WHERE membership.conversation_id = conversation.id AND membership.account_id = reporter.id\n                            AND membership.membership_status = 'active'\n                    )\n                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
        ModerationReportTarget::Media(target_id) => {
            sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
                "INSERT INTO moderation_reports (reporter_account_id, target_media_id, category, description)\n                SELECT reporter.id, target.id, $3, $4 FROM accounts reporter\n                JOIN media_objects target ON target.id = $2\n                WHERE reporter.id = $1 AND reporter.account_status = 'active'\n                    AND reporter.email_verified_at IS NOT NULL\n                    AND (target.owner_account_id = reporter.id\n                        OR (target.resource_type = 'message' AND EXISTS (\n                            SELECT 1 FROM messages message\n                            JOIN conversation_members membership ON membership.conversation_id = message.conversation_id\n                                AND membership.account_id = reporter.id\n                                AND membership.membership_status = 'active'\n                            WHERE message.id = target.resource_id\n                        )))\n                ON CONFLICT DO NOTHING RETURNING id, created_at",
            )
            .bind(reporter_account_id)
            .bind(target_id)
            .bind(category)
            .bind(description)
            .fetch_optional(pool)
            .await
        }
    }
    .map_err(|_| AppError::Internal)?;

    if let Some(record) = inserted {
        return Ok(record);
    }
    if report_exists_for_target(pool, reporter_account_id, target, category).await? {
        return Err(ModerationError::DuplicateActiveReport.into());
    }
    Err(ModerationError::ReportTargetUnavailable.into())
}

async fn report_exists_for_target(
    pool: &PgPool,
    reporter_account_id: Uuid,
    target: ModerationReportTarget,
    category: &str,
) -> Result<bool, AppError> {
    let (column, target_id) = target_database_column(target);
    let query = format!(
        "SELECT EXISTS (SELECT 1 FROM moderation_reports WHERE reporter_account_id = $1 AND {column} = $2 AND category = $3 AND status IN ('open', 'assigned'))"
    );
    sqlx::query_scalar(&query)
        .bind(reporter_account_id)
        .bind(target_id)
        .bind(category)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Internal)
}

fn target_database_column(target: ModerationReportTarget) -> (&'static str, Uuid) {
    match target {
        ModerationReportTarget::Account(id) => ("target_account_id", id),
        ModerationReportTarget::Community(id) => ("target_community_id", id),
        ModerationReportTarget::Post(id) => ("target_post_id", id),
        ModerationReportTarget::Comment(id) => ("target_comment_id", id),
        ModerationReportTarget::ResearchProject(id) => ("target_research_project_id", id),
        ModerationReportTarget::ResearchFinding(id) => ("target_research_finding_id", id),
        ModerationReportTarget::Conversation(id) => ("target_conversation_id", id),
        ModerationReportTarget::Message(id) => ("target_message_id", id),
        ModerationReportTarget::Media(id) => ("target_media_id", id),
    }
}

pub async fn list_reports(
    pool: &PgPool,
    status: Option<ModerationReportStatus>,
    limit: u32,
    offset: u32,
) -> Result<(Vec<ModerationReportPage>, i64), AppError> {
    let status = status.map(ModerationReportStatus::as_database_value);
    let rows = sqlx::query_as::<_, ModerationReportPage>(
        "SELECT id, reporter_account_id,
            CASE WHEN target_account_id IS NOT NULL THEN 'account'
                WHEN target_community_id IS NOT NULL THEN 'community'
                WHEN target_post_id IS NOT NULL THEN 'post'
                WHEN target_comment_id IS NOT NULL THEN 'comment'
                WHEN target_research_project_id IS NOT NULL THEN 'research_project'
                ELSE 'research_finding' END AS target_type,
            COALESCE(target_account_id, target_community_id, target_post_id, target_comment_id,
                target_research_project_id, target_research_finding_id) AS target_id,
            category, description, status, assigned_administrator_id, created_at,
            count(*) OVER() AS total_count
        FROM moderation_reports WHERE ($1::TEXT IS NULL OR status = $1)
        ORDER BY created_at ASC, id ASC LIMIT $2 OFFSET $3",
    )
    .bind(status)
    .bind(i64::from(limit))
    .bind(i64::from(offset))
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let total_count = rows.first().map(|row| row.total_count).unwrap_or(0);
    Ok((rows, total_count))
}

pub async fn resolve_report(
    pool: &PgPool,
    report_id: Uuid,
    administrator_id: Uuid,
    decision: ModerationReportDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM moderation_reports WHERE id = $1 FOR UPDATE",
    )
    .bind(report_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ModerationError::ReportNotFound)?;
    if matches!(status.as_str(), "resolved" | "dismissed") {
        return Err(ModerationError::ReportAlreadyClosed.into());
    }

    let next_status = decision.as_database_value();
    sqlx::query(
        "UPDATE moderation_reports SET status = $2, assigned_administrator_id = COALESCE(assigned_administrator_id, $3),
            resolved_by_administrator_id = $3, resolution_reason = $4, resolved_at = now()
        WHERE id = $1",
    )
    .bind(report_id)
    .bind(next_status)
    .bind(administrator_id)
    .bind(reason)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    record_report_resolution_audit(
        &mut transaction,
        report_id,
        administrator_id,
        decision,
        reason,
        correlation_id,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn record_report_resolution_audit(
    transaction: &mut Transaction<'_, Postgres>,
    report_id: Uuid,
    administrator_id: Uuid,
    decision: ModerationReportDecision,
    reason: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let action = match decision {
        ModerationReportDecision::Resolve => MODERATION_REPORT_RESOLVED_AUDIT_ACTION,
        ModerationReportDecision::Dismiss => MODERATION_REPORT_DISMISSED_AUDIT_ACTION,
    };
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id, result, reason, correlation_id)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, 'success', $5, $6)",
    )
    .bind(administrator_id)
    .bind(action)
    .bind(MODERATION_REPORT_AUDIT_TARGET_TYPE)
    .bind(report_id)
    .bind(reason)
    .bind(correlation_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn assign_report(
    pool: &PgPool,
    report_id: Uuid,
    assigned_administrator_id: Uuid,
    actor_administrator_id: Uuid,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let report_status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM moderation_reports WHERE id = $1 FOR UPDATE",
    )
    .bind(report_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ModerationError::ReportNotFound)?;
    if matches!(report_status.as_str(), "resolved" | "dismissed") {
        return Err(ModerationError::ReportAlreadyClosed.into());
    }

    let assignee_can_resolve = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM administrators administrator
            JOIN administrator_roles assignment ON assignment.administrator_id = administrator.id
                AND assignment.revoked_at IS NULL
            JOIN roles role ON role.role_key = assignment.role_key AND role.is_active = TRUE
            JOIN role_permissions permission ON permission.role_key = role.role_key
            WHERE administrator.id = $1 AND administrator.status = 'active'
                AND permission.permission_key = $2
        )",
    )
    .bind(assigned_administrator_id)
    .bind(crate::constants::moderation::ADMIN_REPORT_RESOLVE_PERMISSION)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !assignee_can_resolve {
        return Err(ModerationError::AdministratorUnavailable.into());
    }

    sqlx::query(
        "UPDATE moderation_reports SET status = $3, assigned_administrator_id = $2
        WHERE id = $1",
    )
    .bind(report_id)
    .bind(assigned_administrator_id)
    .bind(ModerationReportStatus::Assigned.as_database_value())
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let assignment_reason =
        format!("{MODERATION_REPORT_ASSIGNMENT_REASON_PREFIX}{assigned_administrator_id}");
    sqlx::query(
        "INSERT INTO administrative_audit_logs
            (id, actor_administrator_id, action, target_type, target_id, result, reason, correlation_id)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, 'success', $5, $6)",
    )
    .bind(actor_administrator_id)
    .bind(MODERATION_REPORT_ASSIGNED_AUDIT_ACTION)
    .bind(MODERATION_REPORT_AUDIT_TARGET_TYPE)
    .bind(report_id)
    .bind(assignment_reason)
    .bind(correlation_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}
