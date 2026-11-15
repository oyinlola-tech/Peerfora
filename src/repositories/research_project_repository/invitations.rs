use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::research::{ResearchProjectError, ResearchProjectMemberRole},
    errors::app_error::AppError,
    models::research_project::ResearchInvitation,
};

pub async fn invite_research_project_member(
    pool: &PgPool,
    project_id: Uuid,
    inviter_account_id: Uuid,
    invitee_username: &str,
    role: ResearchProjectMemberRole,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let inviter_actor_id = find_account_actor(&mut transaction, inviter_account_id).await?;
    ensure_project_manager(&mut transaction, project_id, inviter_actor_id).await?;
    let invitee_actor_id =
        find_invitee_actor(&mut transaction, invitee_username, inviter_actor_id).await?;
    let inserted = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO research_project_members (
            project_id, actor_id, member_role, membership_status,
            invited_by_actor_id, invited_at
        ) VALUES ($1, $2, $3, 'invited', $4, now())
        ON CONFLICT (project_id, actor_id) DO UPDATE SET
            member_role = EXCLUDED.member_role,
            membership_status = 'invited',
            invited_by_actor_id = EXCLUDED.invited_by_actor_id,
            invited_at = now(), joined_at = NULL, updated_at = now()
        WHERE research_project_members.membership_status IN ('rejected', 'removed', 'left')
        RETURNING actor_id",
    )
    .bind(project_id)
    .bind(invitee_actor_id)
    .bind(role.as_database_value())
    .bind(inviter_actor_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if inserted.is_none() {
        return Err(ResearchProjectError::InvitationAlreadyExists.into());
    }
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_my_research_invitations(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Vec<ResearchInvitation>, AppError> {
    let actor_id: Uuid =
        sqlx::query_scalar("SELECT actor_id FROM human_actor_identities WHERE account_id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| AppError::Internal)?
            .ok_or(ResearchProjectError::InviteeUnavailable)?;
    super::queries::list_invitations_for_actor(pool, actor_id).await
}

async fn find_account_actor(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT actor.actor_id FROM human_actor_identities actor
        JOIN accounts account ON account.id = actor.account_id
        WHERE account.id = $1 AND account.account_status = 'active'
        FOR UPDATE OF account",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchProjectError::ProjectNotFound.into())
}

async fn ensure_project_manager(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let project_status: Option<String> =
        sqlx::query_scalar("SELECT project_status FROM research_projects WHERE id = $1 FOR UPDATE")
            .bind(project_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    match project_status.as_deref() {
        None => return Err(ResearchProjectError::ProjectNotFound.into()),
        Some("active") => {}
        Some(_) => return Err(ResearchProjectError::ProjectUnavailable.into()),
    }

    let can_invite: bool = sqlx::query_scalar(
        "SELECT COALESCE(membership.membership_status = 'active'
            AND membership.member_role IN ('owner', 'supervisor'), FALSE)
        FROM research_projects project
        LEFT JOIN research_project_members membership
            ON membership.project_id = project.id AND membership.actor_id = $2
        WHERE project.id = $1",
    )
    .bind(project_id)
    .bind(actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .unwrap_or(false);
    if can_invite {
        Ok(())
    } else {
        Err(ResearchProjectError::PermissionDenied.into())
    }
}

async fn find_invitee_actor(
    transaction: &mut Transaction<'_, Postgres>,
    username: &str,
    inviter_actor_id: Uuid,
) -> Result<Uuid, AppError> {
    let actor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT identity.actor_id
        FROM accounts account
        JOIN human_actor_identities identity ON identity.account_id = account.id
        WHERE lower(account.username) = lower($1)
            AND account.account_status = 'active'
            AND account.email_verified_at IS NOT NULL",
    )
    .bind(username)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    match actor_id {
        Some(actor_id) if actor_id != inviter_actor_id => Ok(actor_id),
        _ => Err(ResearchProjectError::InviteeUnavailable.into()),
    }
}
