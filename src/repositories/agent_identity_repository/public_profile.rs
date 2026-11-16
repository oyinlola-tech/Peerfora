use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::agents,
    enums::agent::AgentReviewStatus,
    errors::app_error::AppError,
    models::agent_identity::{PublicAgentContributionRecord, PublicAgentProfileRecord},
};

pub async fn find_public_agent_profile_by_handle(
    pool: &PgPool,
    handle: &str,
) -> Result<Option<PublicAgentProfileRecord>, AppError> {
    let profile = sqlx::query_as::<_, PublicAgentProfileRecord>(
        "SELECT agent.actor_id AS id, agent.handle, agent.display_name, agent.description,
            agent.model_disclosure, agent.model_name, agent.model_version,
            agent.supported_languages,
            CASE WHEN institution.id IS NOT NULL THEN agent.institution_directory_id END
                AS institution_directory_id,
            institution.display_name AS institution_directory_name,
            institution.verification_status AS institution_verification_status,
            operator.username AS operator_username, agent.review_status,
            agent.autonomous_research_enabled, agent.autonomous_publishing_enabled,
            ARRAY(
                SELECT specialization.subject_id
                FROM agent_specializations specialization
                WHERE specialization.agent_actor_id = agent.actor_id
                ORDER BY specialization.subject_id
            ) AS specialization_subject_ids,
            ARRAY(
                SELECT capability.capability_key
                FROM agent_capability_requests capability
                WHERE capability.agent_actor_id = agent.actor_id
                    AND capability.request_status = $2
                ORDER BY capability.capability_key
            ) AS approved_capabilities,
            (
                SELECT count(DISTINCT finding.id)
                FROM research_task_submissions submission
                JOIN research_findings finding
                    ON finding.id = submission.promoted_finding_id
                JOIN research_projects project
                    ON project.id = finding.project_id
                WHERE submission.submitted_by_actor_id = agent.actor_id
                    AND submission.review_status = $3
                    AND finding.finding_status = $4
                    AND project.visibility = $5
                    AND project.project_status = $6
            ) AS published_contribution_count
         FROM agent_actor_identities agent
         JOIN accounts operator ON operator.id = agent.operator_account_id
         LEFT JOIN institutions institution
            ON institution.id = agent.institution_directory_id AND institution.is_active
         WHERE lower(agent.handle) = lower($1)
            AND agent.review_status = ANY($7::text[])
            AND operator.account_status = $8
            AND operator.email_verified_at IS NOT NULL",
    )
    .bind(handle)
    .bind(crate::enums::agent::AgentCapabilityReviewStatus::Approved.as_database_value())
    .bind(crate::enums::research::ResearchTaskReviewDecision::Approved.as_database_value())
    .bind(crate::enums::research::ResearchFindingStatus::Published.as_database_value())
    .bind(crate::enums::research::ResearchProjectVisibility::Public.as_database_value())
    .bind(crate::enums::research::ResearchProjectStatus::Active.as_database_value())
    .bind([
        AgentReviewStatus::Approved.as_database_value(),
        AgentReviewStatus::Active.as_database_value(),
    ])
    .bind(crate::enums::account_lifecycle::AccountLifecycleStatus::Active.as_database_value())
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(profile)
}

pub async fn list_public_agent_contributions(
    pool: &PgPool,
    agent_actor_id: Uuid,
) -> Result<Vec<PublicAgentContributionRecord>, AppError> {
    sqlx::query_as::<_, PublicAgentContributionRecord>(
        "SELECT finding.id AS finding_id, finding.project_id,
            published_revision.title, submission.submitted_at AS contributed_at
         FROM research_task_submissions submission
         JOIN research_findings finding
            ON finding.id = submission.promoted_finding_id
         JOIN research_projects project
            ON project.id = finding.project_id
         JOIN research_finding_revisions published_revision
            ON published_revision.finding_id = finding.id
            AND published_revision.revision_number = finding.published_revision_number
         WHERE submission.submitted_by_actor_id = $1
            AND submission.review_status = $2
            AND finding.finding_status = $3
            AND project.visibility = $4
            AND project.project_status = $5
         ORDER BY submission.submitted_at DESC, finding.id
         LIMIT $6",
    )
    .bind(agent_actor_id)
    .bind(crate::enums::research::ResearchTaskReviewDecision::Approved.as_database_value())
    .bind(crate::enums::research::ResearchFindingStatus::Published.as_database_value())
    .bind(crate::enums::research::ResearchProjectVisibility::Public.as_database_value())
    .bind(crate::enums::research::ResearchProjectStatus::Active.as_database_value())
    .bind(agents::MAX_AGENT_PUBLIC_CONTRIBUTIONS)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
