use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    models::discussion::{CommunityPost, CommunityPostComment},
};

pub async fn create_community_post(
    pool: &PgPool,
    community_slug: &str,
    account_id: Uuid,
    post_kind: &str,
    title: &str,
    body: &str,
) -> Result<CommunityPost, AppError> {
    let created_post_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO community_posts (
            community_id, author_actor_id, post_kind, title, body
        )
        SELECT community.id, human.actor_id, $3, $4, $5
        FROM communities community
        JOIN human_actor_identities human ON human.account_id = $2
        JOIN accounts author ON author.id = human.account_id
        JOIN community_members membership ON membership.community_id = community.id
            AND membership.account_id = author.id AND membership.membership_status = 'active'
        WHERE lower(community.slug) = lower($1)
            AND community.lifecycle_status = 'active'
            AND author.account_status = 'active' AND author.email_verified_at IS NOT NULL
        RETURNING id",
    )
    .bind(community_slug)
    .bind(account_id)
    .bind(post_kind)
    .bind(title)
    .bind(body)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    let Some(created_post_id) = created_post_id else {
        let author_can_post = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM accounts
                WHERE id = $1 AND account_status = 'active' AND email_verified_at IS NOT NULL
            )",
        )
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Internal)?;
        if !author_can_post {
            return Err(crate::enums::discussion::DiscussionError::AccountCannotPost.into());
        }
        let community_is_visible = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM communities community
                WHERE lower(community.slug) = lower($1)
                    AND community.lifecycle_status = 'active'
                    AND (community.visibility = 'public' OR EXISTS (
                        SELECT 1 FROM community_members membership
                        WHERE membership.community_id = community.id
                            AND membership.account_id = $2
                            AND membership.membership_status = 'active'
                    ))
            )",
        )
        .bind(community_slug)
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Internal)?;
        let error = if community_is_visible {
            crate::enums::discussion::DiscussionError::MembershipRequired
        } else {
            crate::enums::discussion::DiscussionError::CommunityNotFound
        };
        return Err(error.into());
    };

    sqlx::query_as::<_, CommunityPost>(
        "SELECT post.id, community.slug AS community_slug, author.username AS author_username,
            post.post_kind AS kind, post.title, post.body, post.created_at, post.updated_at
        FROM community_posts post
        JOIN communities community ON community.id = post.community_id
        JOIN human_actor_identities human ON human.actor_id = post.author_actor_id
        JOIN accounts author ON author.id = human.account_id
        WHERE post.id = $1",
    )
    .bind(created_post_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_community_posts_visible_to_account(
    pool: &PgPool,
    community_slug: &str,
    account_id: Option<Uuid>,
    result_limit: i64,
) -> Result<Vec<CommunityPost>, AppError> {
    let community_is_visible = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM communities community
            WHERE lower(community.slug) = lower($1)
                AND community.lifecycle_status = 'active'
                AND (community.visibility = 'public' OR EXISTS (
                    SELECT 1 FROM community_members membership
                    WHERE membership.community_id = community.id
                        AND membership.account_id = $2
                        AND membership.membership_status = 'active'
                ))
        )",
    )
    .bind(community_slug)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    if !community_is_visible {
        return Err(crate::enums::discussion::DiscussionError::CommunityNotFound.into());
    }

    sqlx::query_as::<_, CommunityPost>(
        "SELECT post.id, community.slug AS community_slug, author.username AS author_username,
            post.post_kind AS kind, post.title, post.body, post.created_at, post.updated_at
        FROM communities community
        JOIN community_posts post ON post.community_id = community.id
        JOIN human_actor_identities human ON human.actor_id = post.author_actor_id
        JOIN accounts author ON author.id = human.account_id
        WHERE lower(community.slug) = lower($1) AND community.lifecycle_status = 'active'
            AND post.lifecycle_status = 'active'
            AND (community.visibility = 'public' OR EXISTS (
                SELECT 1 FROM community_members membership
                WHERE membership.community_id = community.id
                    AND membership.account_id = $2 AND membership.membership_status = 'active'
            ))
        ORDER BY post.created_at DESC, post.id DESC LIMIT $3",
    )
    .bind(community_slug)
    .bind(account_id)
    .bind(result_limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn create_post_comment(
    pool: &PgPool,
    post_id: Uuid,
    author_account_id: Uuid,
    parent_comment_id: Option<Uuid>,
    body: &str,
) -> Result<CommunityPostComment, AppError> {
    let created_comment_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO community_post_comments (
            post_id, parent_comment_id, author_actor_id, body
        )
        SELECT post.id, parent.id, human.actor_id, $4
        FROM community_posts post
        JOIN communities community ON community.id = post.community_id
        JOIN human_actor_identities human ON human.account_id = $2
        JOIN accounts author ON author.id = human.account_id
        JOIN community_members membership ON membership.community_id = community.id
            AND membership.account_id = author.id AND membership.membership_status = 'active'
        LEFT JOIN community_post_comments parent ON parent.post_id = post.id
            AND parent.id = $3 AND parent.lifecycle_status = 'active'
        WHERE post.id = $1 AND post.lifecycle_status = 'active'
            AND community.lifecycle_status = 'active'
            AND author.account_status = 'active' AND author.email_verified_at IS NOT NULL
            AND ($3::uuid IS NULL OR parent.id IS NOT NULL)
        RETURNING id",
    )
    .bind(post_id)
    .bind(author_account_id)
    .bind(parent_comment_id)
    .bind(body)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    let Some(created_comment_id) = created_comment_id else {
        return resolve_post_comment_creation_failure(
            pool,
            post_id,
            author_account_id,
            parent_comment_id,
        )
        .await;
    };

    find_post_comment(pool, created_comment_id).await
}

async fn resolve_post_comment_creation_failure(
    pool: &PgPool,
    post_id: Uuid,
    author_account_id: Uuid,
    parent_comment_id: Option<Uuid>,
) -> Result<CommunityPostComment, AppError> {
    let author_can_post = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM accounts
            WHERE id = $1 AND account_status = 'active' AND email_verified_at IS NOT NULL
        )",
    )
    .bind(author_account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if !author_can_post {
        return Err(crate::enums::discussion::DiscussionError::AccountCannotPost.into());
    }

    let post_access = sqlx::query_as::<_, (String, bool)>(
        "SELECT post.lifecycle_status, EXISTS (
            SELECT 1 FROM community_members membership
            WHERE membership.community_id = community.id
                AND membership.account_id = $2 AND membership.membership_status = 'active'
        ) AS is_active_member
        FROM community_posts post
        JOIN communities community ON community.id = post.community_id
        WHERE post.id = $1 AND post.lifecycle_status IN ('active', 'locked')
            AND community.lifecycle_status = 'active'
            AND (community.visibility = 'public' OR EXISTS (
                SELECT 1 FROM community_members membership
                WHERE membership.community_id = community.id
                    AND membership.account_id = $2 AND membership.membership_status = 'active'
            ))",
    )
    .bind(post_id)
    .bind(author_account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(crate::enums::discussion::DiscussionError::PostNotFound)?;

    if post_access.0 == "locked" {
        return Err(crate::enums::discussion::DiscussionError::PostLocked.into());
    }
    if !post_access.1 {
        return Err(crate::enums::discussion::DiscussionError::MembershipRequired.into());
    }

    if let Some(parent_comment_id) = parent_comment_id {
        let parent_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM community_post_comments
                WHERE id = $1 AND post_id = $2 AND lifecycle_status = 'active'
            )",
        )
        .bind(parent_comment_id)
        .bind(post_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::Internal)?;
        if !parent_exists {
            return Err(crate::enums::discussion::DiscussionError::ParentCommentNotFound.into());
        }
    }

    Err(crate::enums::discussion::DiscussionError::PostNotFound.into())
}

pub async fn list_post_comments_visible_to_account(
    pool: &PgPool,
    post_id: Uuid,
    viewer_account_id: Option<Uuid>,
    result_limit: i64,
    result_offset: i64,
) -> Result<Vec<CommunityPostComment>, AppError> {
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
    .bind(viewer_account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if !post_is_visible {
        return Err(crate::enums::discussion::DiscussionError::PostNotFound.into());
    }

    sqlx::query_as::<_, CommunityPostComment>(
        "SELECT comment.id, comment.post_id, comment.parent_comment_id,
            author.username AS author_username, comment.body, comment.created_at,
            comment.updated_at
        FROM community_post_comments comment
        JOIN human_actor_identities human ON human.actor_id = comment.author_actor_id
        JOIN accounts author ON author.id = human.account_id
        WHERE comment.post_id = $1 AND comment.lifecycle_status = 'active'
        ORDER BY comment.created_at, comment.id
        LIMIT $2 OFFSET $3",
    )
    .bind(post_id)
    .bind(result_limit)
    .bind(result_offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

async fn find_post_comment(
    pool: &PgPool,
    comment_id: Uuid,
) -> Result<CommunityPostComment, AppError> {
    sqlx::query_as::<_, CommunityPostComment>(
        "SELECT comment.id, comment.post_id, comment.parent_comment_id,
            author.username AS author_username, comment.body, comment.created_at,
            comment.updated_at
        FROM community_post_comments comment
        JOIN human_actor_identities human ON human.actor_id = comment.author_actor_id
        JOIN accounts author ON author.id = human.account_id
        WHERE comment.id = $1",
    )
    .bind(comment_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}
