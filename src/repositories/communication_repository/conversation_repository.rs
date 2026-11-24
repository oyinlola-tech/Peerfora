use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    constants::phase_four::{
        CONVERSATION_MAX_MEMBERS, CONVERSATION_MAX_PAGE_OFFSET, CONVERSATION_MAX_PAGE_SIZE,
        CONVERSATION_MAX_TITLE_LENGTH,
    },
    enums::account_lifecycle::AccountLifecycleStatus,
    enums::communication::{
        ConversationLifecycleStatus, ConversationMemberRole, ConversationType, MembershipStatus,
    },
    enums::phase_four::CommunicationError,
    enums::research::ResearchProjectStatus,
    errors::app_error::AppError,
    models::communication::{ConversationMemberSummary, ConversationSummary},
};

pub fn validate_conversation_title(title: &Option<String>) -> Result<(), CommunicationError> {
    match title {
        None => Ok(()),
        Some(title) => {
            if title.trim().is_empty() || title.chars().count() > CONVERSATION_MAX_TITLE_LENGTH {
                return Err(CommunicationError::ConversationTitleInvalid);
            }
            Ok(())
        }
    }
}

pub async fn create_conversation(
    pool: &PgPool,
    actor_account_id: Uuid,
    conversation_type: &str,
    title: &Option<String>,
    research_project_id: Option<Uuid>,
    member_account_ids: &[Uuid],
) -> Result<Uuid, AppError> {
    validate_conversation_title(title)?;
    let conversation_type = ConversationType::from_db_value(conversation_type)
        .ok_or(CommunicationError::ConversationTypeInvalid)?;
    if member_account_ids.is_empty()
        || member_account_ids.len() > CONVERSATION_MAX_MEMBERS as usize
        || member_account_ids.contains(&actor_account_id)
    {
        return Err(CommunicationError::ConversationMemberInvalid.into());
    }
    if conversation_type == ConversationType::Direct
        && (member_account_ids.len() != 1 || research_project_id.is_some())
    {
        return Err(CommunicationError::ConversationMemberInvalid.into());
    }

    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let project_membership = match research_project_id {
        Some(project_id) => {
            if conversation_type != ConversationType::ResearchProject {
                return Err(CommunicationError::ConversationTypeInvalid.into());
            }
            let active_account = AccountLifecycleStatus::Active.as_database_value();
            let active_membership = MembershipStatus::Active.as_db_value();
            let active_project = ResearchProjectStatus::Active.as_database_value();
            let permitted: bool = sqlx::query_scalar(
                "SELECT EXISTS (\
                    SELECT 1 FROM research_projects project \
                    JOIN human_actor_identities actor ON actor.account_id = $1 \
                    WHERE project.id = $2 AND project.project_status = $3 \
                        AND actor.actor_status = $4 \
                        AND EXISTS (\
                            SELECT 1 FROM research_project_members member \
                            WHERE member.project_id = project.id AND member.actor_id = actor.actor_id \
                                AND member.membership_status = $5 \
                        )\
                )",
            )
            .bind(actor_account_id)
            .bind(project_id)
            .bind(active_project)
            .bind(active_account)
            .bind(active_membership)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| AppError::Internal)?;
            if !permitted {
                return Err(CommunicationError::ConversationMembershipDenied.into());
            }
            Some(project_id)
        }
        None => None,
    };

    let conversation_id: Uuid = sqlx::query_scalar(
        "INSERT INTO conversations (conversation_type, title, research_project_id, owner_account_id, lifecycle_status, created_by_account_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(conversation_type.as_db_value())
    .bind(title.clone())
    .bind(project_membership)
    .bind(actor_account_id)
    .bind(active_lifecycle)
    .bind(actor_account_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    add_conversation_members(
        &mut transaction,
        conversation_id,
        &[actor_account_id],
        member_account_ids,
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(conversation_id)
}

async fn add_conversation_members(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    initial_account_ids: &[Uuid],
    additional_account_ids: &[Uuid],
) -> Result<(), AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let owner_role = ConversationMemberRole::Owner.as_db_value();
    let member_role = ConversationMemberRole::Member.as_db_value();
    let active_account = AccountLifecycleStatus::Active.as_database_value();
    for account_id in initial_account_ids {
        let result = sqlx::query(
            "INSERT INTO conversation_members (conversation_id, account_id, member_role, membership_status) SELECT $1, account.id, CASE WHEN account.id = $2 THEN $3 ELSE $4 END, $5 FROM accounts account WHERE account.id = $2 AND account.account_status = $6",
        )
        .bind(conversation_id)
        .bind(account_id)
        .bind(owner_role)
        .bind(member_role)
        .bind(active_membership)
        .bind(active_account)
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
        if result.rows_affected() == 0 {
            return Err(CommunicationError::ConversationMemberInvalid.into());
        }
    }
    if additional_account_ids.is_empty() {
        return Ok(());
    }
    let result = sqlx::query(
        "INSERT INTO conversation_members (conversation_id, account_id, membership_status) SELECT $1, account.id, $3 FROM accounts account WHERE account.id = ANY($2::UUID[]) AND account.account_status = $4 AND NOT EXISTS (SELECT 1 FROM user_blocks block WHERE block.blocker_account_id = account.id OR block.blocked_account_id = account.id) ON CONFLICT DO NOTHING RETURNING account_id",
    )
    .bind(conversation_id)
    .bind(additional_account_ids)
    .bind(active_membership)
    .bind(active_account)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if result.len() != additional_account_ids.len() {
        return Err(CommunicationError::ConversationMemberInvalid.into());
    }
    Ok(())
}

pub async fn list_conversations(
    pool: &PgPool,
    account_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ConversationSummary>, i64), AppError> {
    if !(1..=CONVERSATION_MAX_PAGE_SIZE).contains(&limit)
        || offset.is_negative()
        || offset > CONVERSATION_MAX_PAGE_OFFSET
    {
        return Err(CommunicationError::ConversationMemberInvalid.into());
    }
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let closed_lifecycle = ConversationLifecycleStatus::Closed.as_db_value();
    let rows = sqlx::query_as::<_, ConversationSummary>(
        "SELECT conversation.id, conversation.conversation_type, conversation.title,\
            conversation.research_project_id, conversation.owner_account_id,\
            conversation.lifecycle_status, conversation.created_at, conversation.updated_at,\
            last_message.id AS last_message_id,\
            last_message.created_at AS last_message_created_at,\
            member_counts.member_count,\
            COALESCE(unread.unread_count, 0) AS unread_count,\
            conversation.lifecycle_status <> $3 AS is_restricted,\
            conversation.lifecycle_status = $4 AS is_closed \
         FROM conversations conversation \
         JOIN conversation_members membership ON membership.conversation_id = conversation.id \
         AND membership.account_id = $1 AND membership.membership_status = $2 \
         LEFT JOIN messages last_message ON last_message.id = (\
             SELECT id FROM messages \
             WHERE conversation_id = conversation.id \
             ORDER BY created_at DESC, id DESC LIMIT 1 \
         )\
         JOIN LATERAL (\
             SELECT count(*)::BIGINT AS member_count FROM conversation_members count_member \
             WHERE count_member.conversation_id = conversation.id \
                 AND count_member.membership_status = $2 \
         ) member_counts ON TRUE \
         LEFT JOIN LATERAL (\
             SELECT count(*)::BIGINT AS unread_count FROM messages message \
             WHERE message.conversation_id = conversation.id AND message.sender_account_id <> $1 \
                 AND message.created_at > COALESCE((SELECT COALESCE(MAX(read_at), created_at) FROM message_read_states \
                     WHERE message_id = message.id AND account_id = $1), message.created_at)\
         ) unread ON TRUE \
         ORDER BY COALESCE(last_message.created_at, conversation.updated_at) DESC, conversation.id DESC \
         LIMIT $5 OFFSET $6",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(active_lifecycle)
    .bind(closed_lifecycle)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)::BIGINT FROM conversations conversation \
         JOIN conversation_members membership ON membership.conversation_id = conversation.id \
         AND membership.account_id = $1 AND membership.membership_status = $2",
    )
    .bind(account_id)
    .bind(active_membership)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((rows, total))
}

pub async fn get_conversation(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
) -> Result<ConversationSummary, AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let closed_lifecycle = ConversationLifecycleStatus::Closed.as_db_value();
    sqlx::query_as::<_, ConversationSummary>(
        "SELECT conversation.id, conversation.conversation_type, conversation.title,\
            conversation.research_project_id, conversation.owner_account_id,\
            conversation.lifecycle_status, conversation.created_at, conversation.updated_at,\
            last_message.id AS last_message_id,\
            last_message.created_at AS last_message_created_at,\
            member_counts.member_count,\
            COALESCE(unread.unread_count, 0) AS unread_count,\
            conversation.lifecycle_status <> $3 AS is_restricted,\
            conversation.lifecycle_status = $4 AS is_closed \
         FROM conversations conversation \
         JOIN conversation_members membership ON membership.conversation_id = conversation.id \
         AND membership.account_id = $1 AND membership.membership_status = $2 \
         LEFT JOIN messages last_message ON last_message.id = (\
             SELECT id FROM messages \
             WHERE conversation_id = conversation.id \
             ORDER BY created_at DESC, id DESC LIMIT 1 \
         )\
         JOIN LATERAL (\
             SELECT count(*)::BIGINT AS member_count FROM conversation_members count_member \
             WHERE count_member.conversation_id = conversation.id \
                 AND count_member.membership_status = $2 \
         ) member_counts ON TRUE \
         LEFT JOIN LATERAL (\
             SELECT count(*)::BIGINT AS unread_count FROM messages message \
             WHERE message.conversation_id = conversation.id AND message.sender_account_id <> $1 \
                 AND message.created_at > COALESCE((SELECT COALESCE(MAX(read_at), created_at) FROM message_read_states \
                     WHERE message_id = message.id AND account_id = $1), message.created_at)\
         ) unread ON TRUE \
         WHERE conversation.id = $5",
    )
    .bind(account_id)
    .bind(active_membership)
    .bind(active_lifecycle)
    .bind(closed_lifecycle)
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunicationError::ConversationMembershipDenied.into())
}

pub async fn list_members(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
) -> Result<Vec<ConversationMemberSummary>, AppError> {
    require_active_membership(pool, account_id, conversation_id).await?;
    let active_membership = MembershipStatus::Active.as_db_value();
    sqlx::query_as::<_, ConversationMemberSummary>(
        "SELECT conversation_members.conversation_id, conversation_members.account_id,\
            conversation_members.member_role, conversation_members.membership_status,\
            conversation_members.joined_at \
         FROM conversation_members \
         WHERE conversation_id = $1 AND membership_status = $2 \
         ORDER BY conversation_members.member_role DESC, conversation_members.joined_at ASC",
    )
    .bind(conversation_id)
    .bind(active_membership)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn require_active_membership(
    pool: &PgPool,
    account_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), AppError> {
    let active_membership = MembershipStatus::Active.as_db_value();
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let archived_lifecycle = ConversationLifecycleStatus::Archived.as_db_value();
    let permitted: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
            SELECT 1 FROM conversations conversation \
            JOIN conversation_members membership ON membership.conversation_id = conversation.id \
                AND membership.account_id = $1 AND membership.membership_status = $2 \
            WHERE conversation.id = $3 AND conversation.lifecycle_status IN ($4, $5)\
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

pub async fn set_lifecycle_status(
    pool: &PgPool,
    conversation_id: Uuid,
    status: ConversationLifecycleStatus,
) -> Result<(), AppError> {
    let active_lifecycle = ConversationLifecycleStatus::Active.as_db_value();
    let result = sqlx::query("UPDATE conversations SET lifecycle_status = $1, updated_at = now() WHERE id = $2 AND lifecycle_status = $3")
        .bind(status.as_db_value())
        .bind(conversation_id)
        .bind(active_lifecycle)
        .execute(pool)
        .await;
    if result.is_ok() {
        return Ok(());
    }
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM conversations WHERE id = $1)")
            .bind(conversation_id)
            .fetch_one(pool)
            .await
            .map_err(|_| AppError::Internal)?;
    if exists {
        return Err(CommunicationError::ConversationRestricted.into());
    }
    Err(CommunicationError::ConversationNotFound.into())
}

pub async fn touch_conversation(pool: &PgPool, conversation_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE conversations SET updated_at = now() WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}

pub async fn conversation_timestamp(
    pool: &PgPool,
    conversation_id: Uuid,
) -> Result<DateTime<Utc>, AppError> {
    sqlx::query_scalar::<_, DateTime<Utc>>("SELECT updated_at FROM conversations WHERE id = $1")
        .bind(conversation_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Internal)
}
