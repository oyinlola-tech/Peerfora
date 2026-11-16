use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::discussions::MAX_POST_RESEARCH_FINDING_REFERENCES,
    enums::discussion::DiscussionError, errors::app_error::AppError,
};

pub async fn create_public_research_finding_reference(
    pool: &PgPool,
    post_id: Uuid,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let post_access = sqlx::query_as::<_, (bool, bool)>(
        "SELECT post.author_actor_id = human.actor_id,
            EXISTS (
                SELECT 1 FROM community_members membership
                WHERE membership.community_id = community.id
                    AND membership.account_id = author.id
                    AND membership.membership_status = 'active'
            )
        FROM community_posts post
        JOIN communities community ON community.id = post.community_id
        JOIN accounts author ON author.id = $2
        LEFT JOIN human_actor_identities human ON human.account_id = author.id
        WHERE post.id = $1 AND post.lifecycle_status = 'active'
            AND community.lifecycle_status = 'active'
            AND author.account_status = 'active'
            AND author.email_verified_at IS NOT NULL
        FOR UPDATE OF post",
    )
    .bind(post_id)
    .bind(account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(DiscussionError::PostNotFound)?;
    if !post_access.0 {
        return Err(DiscussionError::PostResearchReferencePermissionDenied.into());
    }
    if !post_access.1 {
        return Err(DiscussionError::MembershipRequired.into());
    }

    let reference_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM community_post_research_findings WHERE post_id = $1",
    )
    .bind(post_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if reference_count >= MAX_POST_RESEARCH_FINDING_REFERENCES {
        return Err(DiscussionError::ResearchFindingReferenceLimitReached.into());
    }

    let inserted: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO community_post_research_findings (post_id, finding_id)
        VALUES ($1, $2)
        ON CONFLICT (post_id, finding_id) DO NOTHING
        RETURNING finding_id",
    )
    .bind(post_id)
    .bind(finding_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if inserted.is_none() {
        return Err(DiscussionError::ResearchFindingAlreadyLinked.into());
    }
    transaction.commit().await.map_err(|_| AppError::Internal)
}

pub async fn list_research_finding_reference_ids_for_visible_post(
    pool: &PgPool,
    post_id: Uuid,
    account_id: Option<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let post_is_visible = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM community_posts post
            JOIN communities community ON community.id = post.community_id
            WHERE post.id = $1 AND post.lifecycle_status IN ('active', 'locked')
                AND community.lifecycle_status = 'active'
                AND (community.visibility = 'public' OR EXISTS (
                    SELECT 1 FROM community_members membership
                    WHERE membership.community_id = community.id
                        AND membership.account_id = $2
                        AND membership.membership_status = 'active'
                ))
        )",
    )
    .bind(post_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if !post_is_visible {
        return Err(DiscussionError::PostNotFound.into());
    }

    sqlx::query_scalar(
        "SELECT reference.finding_id
        FROM community_post_research_findings reference
        WHERE reference.post_id = $1
        ORDER BY reference.created_at, reference.finding_id
        LIMIT $2",
    )
    .bind(post_id)
    .bind(MAX_POST_RESEARCH_FINDING_REFERENCES)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn remove_post_research_finding_reference(
    pool: &PgPool,
    post_id: Uuid,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let post_access = sqlx::query_as::<_, (bool, bool)>(
        "SELECT post.author_actor_id = human.actor_id,
            EXISTS (
                SELECT 1 FROM community_members membership
                WHERE membership.community_id = community.id
                    AND membership.account_id = author.id
                    AND membership.membership_status = 'active'
            )
        FROM community_posts post
        JOIN communities community ON community.id = post.community_id
        JOIN accounts author ON author.id = $2
        LEFT JOIN human_actor_identities human ON human.account_id = author.id
        WHERE post.id = $1 AND post.lifecycle_status = 'active'
            AND community.lifecycle_status = 'active'
            AND author.account_status = 'active'
            AND author.email_verified_at IS NOT NULL
        FOR UPDATE OF post",
    )
    .bind(post_id)
    .bind(account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(DiscussionError::PostNotFound)?;
    if !post_access.0 {
        return Err(DiscussionError::PostResearchReferencePermissionDenied.into());
    }
    if !post_access.1 {
        return Err(DiscussionError::MembershipRequired.into());
    }

    let deleted = sqlx::query(
        "DELETE FROM community_post_research_findings
        WHERE post_id = $1 AND finding_id = $2",
    )
    .bind(post_id)
    .bind(finding_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;
    if deleted.rows_affected() == 0 {
        return Err(DiscussionError::ResearchFindingUnavailable.into());
    }
    transaction.commit().await.map_err(|_| AppError::Internal)
}
