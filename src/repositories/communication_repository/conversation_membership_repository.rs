use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::communication::{ConversationLifecycleStatus, MembershipStatus},
    enums::phase_four::CommunicationError,
    errors::app_error::AppError,
};

pub async fn require_active_membership(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let archived_lifecycle = ConversationLifecycleStatus::Archived.as_db_value();
    let permitted: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM conversations conversation
            JOIN conversation_members membership
              ON membership.conversation_id = conversation.id
             AND membership.account_id = $1
             AND membership.membership_status = $2
            WHERE conversation.id = $3
              AND conversation.lifecycle_status IN ($4, $5)
        )",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(conversation_id)
    .bind(active_lifecycle)
    .bind(archived_lifecycle)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if permitted {
        Ok(())
    } else {
        Err(CommunicationError::ConversationMembershipDenied.into())
    }
}

pub async fn update_conversation_title(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    title: &Option<String>,
) -> Result<(), AppError> {
    let active = ConversationLifecycleStatus::Active.as_db_value();
    let archived = ConversationLifecycleStatus::Archived.as_db_value();
    sqlx::query(
        "UPDATE conversations SET title = $3, updated_at = now() \
         WHERE id = $1 AND owner_account_id = $2 \
           AND lifecycle_status IN ($4, $5) RETURNING id",
    )
    .bind(conversation_id)
    .bind(account_id)
    .bind(title)
    .bind(active)
    .bind(archived)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::ConversationMembershipDenied)?;
    Ok(())
}

pub async fn add_conversation_members(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    account_ids: &[Uuid],
) -> Result<(), AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let project_id = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT research_project_id FROM conversations \
         WHERE id = $1 AND owner_account_id = $2 AND lifecycle_status = $3 \
         FOR UPDATE",
    )
    .bind(conversation_id)
    .bind(account_id)
    .bind(active_lifecycle)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::ConversationMembershipDenied)?;
    let member_is_active: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM conversation_members
            WHERE conversation_id = $1 AND account_id = $2 AND membership_status = $3
        )",
    )
    .bind(conversation_id)
    .bind(account_id)
    .bind(active_membership)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !member_is_active {
        return Err(CommunicationError::ConversationMembershipDenied.into());
    }
    super::conversation_repository::add_conversation_members(
        &mut transaction,
        conversation_id,
        &[],
        account_ids,
        project_id,
    )
    .await?;
    sqlx::query("UPDATE conversations SET updated_at = now() WHERE id = $1")
        .bind(conversation_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn remove_conversation_member(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
    target_account_id: Uuid,
) -> Result<(), AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let removed_membership = MembershipStatus::Removed.as_db_value();
    let mut transaction: Transaction<'_, Postgres> =
        pool.begin().await.map_err(|_| AppError::Internal)?;
    let permitted: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM conversations conversation
            JOIN conversation_members membership
              ON membership.conversation_id = conversation.id
             AND membership.account_id = $1
             AND membership.membership_status = $2
            WHERE conversation.id = $3
              AND conversation.lifecycle_status = $4
              AND (conversation.owner_account_id = $1 OR $1 = $5)
        )",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(conversation_id)
    .bind(active_lifecycle)
    .bind(target_account_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if !permitted {
        return Err(CommunicationError::ConversationMembershipDenied.into());
    }
    let result = sqlx::query(
        "UPDATE conversation_members SET membership_status = $3, updated_at = now() \
         WHERE conversation_id = $1 AND account_id = $2 \
           AND membership_status = $4 RETURNING conversation_id",
    )
    .bind(conversation_id)
    .bind(target_account_id)
    .bind(removed_membership)
    .bind(active_membership)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.is_none() {
        return Err(CommunicationError::ConversationMembershipDenied.into());
    }
    sqlx::query("UPDATE conversations SET updated_at = now() WHERE id = $1")
        .bind(conversation_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}
