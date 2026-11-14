use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::app_error::AppError, models::community::Community};

pub async fn find_community_visible_to_account(
    pool: &PgPool,
    slug: &str,
    account_id: Option<Uuid>,
) -> Result<Option<Community>, AppError> {
    sqlx::query_as::<_, Community>(
        "SELECT community.id, community.slug, community.display_name,
            community.description, community.category, community.institution_id,
            community.course_id, community.discipline_id, community.owner_account_id,
            community.visibility,
            community.membership_policy, community.lifecycle_status, community.created_at
        FROM communities community
        WHERE lower(community.slug) = lower($1)
            AND community.lifecycle_status = 'active'
            AND (
                community.visibility IN ('public', 'restricted')
                OR EXISTS (
                    SELECT 1 FROM community_members membership
                    WHERE membership.community_id = community.id
                        AND membership.account_id = $2
                        AND membership.membership_status = 'active'
                )
            )",
    )
    .bind(slug)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_discoverable_communities(
    pool: &PgPool,
    result_limit: i64,
) -> Result<Vec<Community>, AppError> {
    sqlx::query_as::<_, Community>(
        "SELECT id, slug, display_name, description, category, institution_id,
            course_id, discipline_id, owner_account_id, visibility, membership_policy,
            lifecycle_status, created_at
        FROM communities
        WHERE visibility IN ('public', 'restricted') AND lifecycle_status = 'active'
        ORDER BY created_at DESC, id DESC
        LIMIT $1",
    )
    .bind(result_limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
