use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::{admin_auth_error::AdminAuthError, community::CommunityLifecycleStatus},
    errors::app_error::AppError,
    models::admin_identity::{AdministrativeCommunityFilters, AdministrativeCommunityRecord},
    repositories::admin_auth_repository::{
        AdministrativeAuditEvent, insert_administrative_audit_event,
    },
};

pub async fn search_communities_for_administration(
    pool: &PgPool,
    filters: &AdministrativeCommunityFilters<'_>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AdministrativeCommunityRecord>, i64), AppError> {
    let total_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM communities community
        WHERE ($1::text IS NULL OR lower(community.display_name) LIKE $1 || '%' ESCAPE E'\\\\'
                OR lower(community.slug) LIKE $1 || '%' ESCAPE E'\\\\')
            AND ($2::text IS NULL OR community.category = $2)
            AND ($3::text IS NULL OR community.lifecycle_status = $3)",
    )
    .bind(filters.search_prefix)
    .bind(filters.category)
    .bind(filters.lifecycle_status)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let records = sqlx::query_as::<_, AdministrativeCommunityRecord>(
        "SELECT community.id, community.slug, community.display_name, community.category,
            community.visibility, community.lifecycle_status, community.owner_account_id,
            count(membership.account_id) FILTER (WHERE membership.membership_status = 'active')
                AS member_count,
            community.created_at
        FROM communities community
        LEFT JOIN community_members membership ON membership.community_id = community.id
        WHERE ($1::text IS NULL OR lower(community.display_name) LIKE $1 || '%' ESCAPE E'\\\\'
                OR lower(community.slug) LIKE $1 || '%' ESCAPE E'\\\\')
            AND ($2::text IS NULL OR community.category = $2)
            AND ($3::text IS NULL OR community.lifecycle_status = $3)
        GROUP BY community.id
        ORDER BY community.created_at DESC, community.id
        LIMIT $4 OFFSET $5",
    )
    .bind(filters.search_prefix)
    .bind(filters.category)
    .bind(filters.lifecycle_status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((records, total_count))
}

pub async fn find_community_for_administration(
    pool: &PgPool,
    community_id: Uuid,
) -> Result<Option<AdministrativeCommunityRecord>, AppError> {
    sqlx::query_as::<_, AdministrativeCommunityRecord>(
        "SELECT community.id, community.slug, community.display_name, community.category,
            community.visibility, community.lifecycle_status, community.owner_account_id,
            count(membership.account_id) FILTER (WHERE membership.membership_status = 'active')
                AS member_count,
            community.created_at
        FROM communities community
        LEFT JOIN community_members membership ON membership.community_id = community.id
        WHERE community.id = $1
        GROUP BY community.id",
    )
    .bind(community_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn change_community_lifecycle_status(
    pool: &PgPool,
    community_id: Uuid,
    administrator_id: Uuid,
    expected_status: CommunityLifecycleStatus,
    new_status: CommunityLifecycleStatus,
    reason_code: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let current_status = lock_community_for_administration(&mut transaction, community_id).await?;
    let current_status = CommunityLifecycleStatus::try_from(current_status.as_str())
        .map_err(|_| AppError::Internal)?;
    if current_status != expected_status {
        return Err(AdminAuthError::CommunityStateConflict.into());
    }
    update_community_lifecycle_status(&mut transaction, community_id, new_status).await?;
    let action = match new_status {
        CommunityLifecycleStatus::Suspended => "community.suspended",
        CommunityLifecycleStatus::Active => "community.restored",
        CommunityLifecycleStatus::Archived => return Err(AppError::Internal),
    };
    insert_administrative_audit_event(
        &mut transaction,
        AdministrativeAuditEvent {
            actor_id: administrator_id,
            action,
            target_type: "community",
            target_id: community_id,
            result: "success",
            reason: Some(reason_code),
            correlation_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(|_| AppError::Internal)
}

async fn lock_community_for_administration(
    transaction: &mut Transaction<'_, Postgres>,
    community_id: Uuid,
) -> Result<String, AppError> {
    sqlx::query_scalar::<_, String>(
        "SELECT lifecycle_status FROM communities WHERE id = $1 FOR UPDATE",
    )
    .bind(community_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?
    .ok_or(AdminAuthError::CommunityNotFound.into())
}

async fn update_community_lifecycle_status(
    transaction: &mut Transaction<'_, Postgres>,
    community_id: Uuid,
    new_status: CommunityLifecycleStatus,
) -> Result<(), AppError> {
    sqlx::query("UPDATE communities SET lifecycle_status = $2, updated_at = now() WHERE id = $1")
        .bind(community_id)
        .bind(new_status.as_database_value())
        .execute(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(())
}
