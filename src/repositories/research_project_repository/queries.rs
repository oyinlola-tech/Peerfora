use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    models::research_project::{ResearchInvitation, ResearchProject},
};

const RESEARCH_PROJECT_SELECT_FIELDS: &str =
    "SELECT project.id, owner_account.username AS owner_username, project.title,
        project.abstract AS abstract_text,
        project.description, project.objectives, project.research_questions,
        ARRAY(
            SELECT subject.subject_id FROM research_project_subjects subject
            WHERE subject.project_id = project.id ORDER BY subject.subject_id
        ) AS subject_ids,
        project.visibility, project.project_status, member.member_role AS current_member_role,
        project.created_at, project.updated_at";

pub async fn find_research_project_visible_to_actor(
    pool: &PgPool,
    project_id: Uuid,
    actor_id: Option<Uuid>,
) -> Result<Option<ResearchProject>, AppError> {
    let query = format!(
        "{RESEARCH_PROJECT_SELECT_FIELDS}
        FROM research_projects project
    JOIN human_actor_identities owner_identity ON owner_identity.actor_id = project.owner_actor_id
    JOIN accounts owner_account ON owner_account.id = owner_identity.account_id
        LEFT JOIN research_project_members member
            ON member.project_id = project.id AND member.actor_id = $2
            AND member.membership_status = 'active'
        WHERE project.id = $1 AND project.project_status = 'active'
            AND (project.visibility = 'public' OR member.actor_id IS NOT NULL)"
    );
    sqlx::query_as::<_, ResearchProject>(&query)
        .bind(project_id)
        .bind(actor_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn list_research_projects_visible_to_actor(
    pool: &PgPool,
    actor_id: Uuid,
    result_limit: i64,
) -> Result<Vec<ResearchProject>, AppError> {
    let query = format!(
        "{RESEARCH_PROJECT_SELECT_FIELDS}
        FROM research_projects project
        JOIN human_actor_identities owner_identity ON owner_identity.actor_id = project.owner_actor_id
        JOIN accounts owner_account ON owner_account.id = owner_identity.account_id
        LEFT JOIN research_project_members member
            ON member.project_id = project.id AND member.actor_id = $1
            AND member.membership_status = 'active'
        WHERE project.project_status = 'active'
            AND (project.visibility = 'public' OR member.actor_id IS NOT NULL)
        ORDER BY project.updated_at DESC, project.id
        LIMIT $2"
    );
    sqlx::query_as::<_, ResearchProject>(&query)
        .bind(actor_id)
        .bind(result_limit)
        .fetch_all(pool)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn list_invitations_for_actor(
    pool: &PgPool,
    actor_id: Uuid,
) -> Result<Vec<ResearchInvitation>, AppError> {
    sqlx::query_as::<_, ResearchInvitation>(
        "SELECT project.id AS project_id, project.title AS project_title,
            inviter_account.username AS invited_by_username, member.member_role AS role,
            member.invited_at
        FROM research_project_members member
        JOIN research_projects project ON project.id = member.project_id
        JOIN human_actor_identities inviter ON inviter.actor_id = member.invited_by_actor_id
        JOIN accounts inviter_account ON inviter_account.id = inviter.account_id
        WHERE member.actor_id = $1 AND member.membership_status = 'invited'
            AND project.project_status = 'active'
        ORDER BY member.invited_at DESC, project.id",
    )
    .bind(actor_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
