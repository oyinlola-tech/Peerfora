use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{enums::research::ResearchProjectError, errors::app_error::AppError};

pub async fn accept_research_project_invitation(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    change_invitation_status(pool, project_id, account_id, "active").await
}

pub async fn reject_research_project_invitation(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    change_invitation_status(pool, project_id, account_id, "rejected").await
}

async fn change_invitation_status(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    new_status: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id = find_actor_for_active_account(&mut transaction, account_id).await?;
    lock_active_project(&mut transaction, project_id).await?;
    let invitation_updated = sqlx::query(
        "UPDATE research_project_members
        SET membership_status = $3,
            joined_at = CASE WHEN $3 = 'active' THEN now() ELSE joined_at END,
            updated_at = now()
        WHERE project_id = $1 AND actor_id = $2 AND membership_status = 'invited'",
    )
    .bind(project_id)
    .bind(actor_id)
    .bind(new_status)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .rows_affected()
        == 1;
    if !invitation_updated {
        return Err(ResearchProjectError::InvitationNotFound.into());
    }
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn leave_research_project(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id = find_actor_for_active_account(&mut transaction, account_id).await?;
    lock_active_project(&mut transaction, project_id).await?;
    let membership = sqlx::query_scalar::<_, String>(
        "UPDATE research_project_members SET membership_status = 'left', updated_at = now()
        WHERE project_id = $1 AND actor_id = $2 AND membership_status = 'active'
        RETURNING member_role",
    )
    .bind(project_id)
    .bind(actor_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    match membership.as_deref() {
        Some("owner") => return Err(ResearchProjectError::OwnerCannotLeave.into()),
        Some(_) => {}
        None => return Err(ResearchProjectError::MemberNotFound.into()),
    }
    cancel_open_tasks_assigned_to_member(&mut transaction, project_id, actor_id).await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn remove_research_project_member(
    pool: &PgPool,
    project_id: Uuid,
    owner_account_id: Uuid,
    username: &str,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let owner_actor_id = find_actor_for_active_account(&mut transaction, owner_account_id).await?;
    lock_active_project(&mut transaction, project_id).await?;
    ensure_actor_owns_project(&mut transaction, project_id, owner_actor_id).await?;
    let target_actor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT identity.actor_id FROM human_actor_identities identity
        JOIN accounts account ON account.id = identity.account_id
        WHERE lower(account.username) = lower($1)",
    )
    .bind(username)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    let target_actor_id = target_actor_id.ok_or(ResearchProjectError::MemberNotFound)?;
    if target_actor_id == owner_actor_id {
        return Err(ResearchProjectError::CannotRemoveProjectOwner.into());
    }
    cancel_open_tasks_assigned_to_member(&mut transaction, project_id, target_actor_id).await?;
    let removed = sqlx::query(
        "UPDATE research_project_members
        SET membership_status = 'removed', updated_at = now()
        WHERE project_id = $1 AND actor_id = $2
            AND membership_status IN ('active', 'invited')",
    )
    .bind(project_id)
    .bind(target_actor_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .rows_affected()
        == 1;
    if !removed {
        return Err(ResearchProjectError::MemberNotFound.into());
    }
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn cancel_open_tasks_assigned_to_member(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE research_tasks SET task_status = 'cancelled', cancelled_at = now(),
            updated_at = now()
        WHERE project_id = $1 AND assigned_actor_id = $2
            AND task_status NOT IN ('completed', 'failed', 'cancelled')",
    )
    .bind(project_id)
    .bind(actor_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok(())
}

async fn find_actor_for_active_account(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar(
        "SELECT identity.actor_id FROM human_actor_identities identity
        JOIN accounts account ON account.id = identity.account_id
        WHERE account.id = $1 AND account.account_status = 'active'
        FOR UPDATE OF account",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(ResearchProjectError::ProjectNotFound.into())
}

async fn lock_active_project(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<(), AppError> {
    let project_status: Option<String> =
        sqlx::query_scalar("SELECT project_status FROM research_projects WHERE id = $1 FOR UPDATE")
            .bind(project_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    match project_status.as_deref() {
        Some("active") => Ok(()),
        Some(_) => Err(ResearchProjectError::ProjectUnavailable.into()),
        None => Err(ResearchProjectError::ProjectNotFound.into()),
    }
}

async fn ensure_actor_owns_project(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    actor_id: Uuid,
) -> Result<(), AppError> {
    let is_owner: bool =
        sqlx::query_scalar("SELECT owner_actor_id = $2 FROM research_projects WHERE id = $1")
            .bind(project_id)
            .bind(actor_id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|_| AppError::Internal)?;
    if is_owner {
        Ok(())
    } else {
        Err(ResearchProjectError::PermissionDenied.into())
    }
}
