use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::community_error::CommunityError, errors::app_error::AppError,
    models::community::CommunityMembership,
};

use super::audit::record_community_change;

pub async fn join_community(
    pool: &PgPool,
    slug: &str,
    account_id: Uuid,
) -> Result<CommunityMembership, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let community = sqlx::query_as::<_, (Uuid, String, String, String)>(
        "SELECT id, membership_policy, lifecycle_status, visibility
        FROM communities WHERE lower(slug) = lower($1) FOR UPDATE",
    )
    .bind(slug)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunityError::CommunityNotFound)?;

    if community.2 != "active" {
        return Err(CommunityError::MembershipNotAllowed.into());
    }

    let current_membership = sqlx::query_as::<_, (String, String)>(
        "SELECT membership_status, community_role FROM community_members
         WHERE community_id = $1 AND account_id = $2",
    )
    .bind(community.0)
    .bind(account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    if community.3 == "private"
        && !current_membership
            .as_ref()
            .is_some_and(|(status, _)| matches!(status.as_str(), "active" | "invited"))
    {
        return Err(CommunityError::CommunityNotFound.into());
    }

    if current_membership
        .as_ref()
        .is_some_and(|(status, _)| status == "banned")
    {
        return Err(CommunityError::MembershipBanned.into());
    }

    let was_invited = current_membership
        .as_ref()
        .is_some_and(|(status, _)| status == "invited");
    let membership_status = if community.1 == "approval_required" && !was_invited {
        "requested"
    } else {
        "active"
    };
    let joined_at = (membership_status == "active").then_some(chrono::Utc::now());

    let membership = sqlx::query_as::<_, CommunityMembership>(
        "INSERT INTO community_members (
            community_id, account_id, membership_status, community_role, joined_at
        ) VALUES ($1, $2, $3, 'member', $4)
        ON CONFLICT (community_id, account_id) DO UPDATE
        SET membership_status = CASE
                WHEN community_members.membership_status = 'active' THEN 'active'
                ELSE EXCLUDED.membership_status
            END,
            community_role = CASE
                WHEN community_members.community_role = 'owner' THEN 'owner'
                ELSE 'member'
            END,
            joined_at = CASE
                WHEN community_members.membership_status = 'active'
                    THEN community_members.joined_at
                ELSE EXCLUDED.joined_at
            END,
            updated_at = now()
        RETURNING community_id, account_id, membership_status, community_role, joined_at",
    )
    .bind(community.0)
    .bind(account_id)
    .bind(membership_status)
    .bind(joined_at)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    let previous_status = current_membership
        .as_ref()
        .map(|(status, _)| status.as_str());
    let membership_was_changed = previous_status != Some(membership.membership_status.as_str());
    if membership_was_changed {
        let event_type = if membership.membership_status == "active" {
            "community_member_joined"
        } else {
            "community_membership_requested"
        };
        record_community_change(
            &mut transaction,
            community.0,
            account_id,
            Some(account_id),
            event_type,
            serde_json::json!({"membershipStatus": membership.membership_status}),
        )
        .await?;
    }

    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(membership)
}

pub async fn leave_community(pool: &PgPool, slug: &str, account_id: Uuid) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let membership = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT community.id, community_members.membership_status,
            community_members.community_role
        FROM communities community
        JOIN community_members ON community_members.community_id = community.id
        WHERE lower(community.slug) = lower($1)
            AND community_members.account_id = $2
        FOR UPDATE OF community, community_members",
    )
    .bind(slug)
    .bind(account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(CommunityError::CommunityNotFound)?;

    if membership.2 == "owner" {
        return Err(CommunityError::OwnerCannotLeave.into());
    }

    let updated_rows = sqlx::query(
        "UPDATE community_members
        SET membership_status = 'left', community_role = 'member',
            joined_at = NULL, updated_at = now()
        WHERE community_id = $1 AND account_id = $2
            AND membership_status IN ('active', 'requested')",
    )
    .bind(membership.0)
    .bind(account_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .rows_affected();

    if updated_rows > 0 {
        record_community_change(
            &mut transaction,
            membership.0,
            account_id,
            Some(account_id),
            "community_member_left",
            serde_json::json!({}),
        )
        .await?;
    }

    transaction.commit().await.map_err(|_| AppError::Internal)
}
