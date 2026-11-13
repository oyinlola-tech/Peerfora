use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::community_error::CommunityError,
    errors::app_error::AppError,
    models::community::{Community, CommunityMembership},
};

pub struct NewCommunity<'a> {
    pub slug: &'a str,
    pub display_name: &'a str,
    pub description: &'a str,
    pub category: &'a str,
    pub visibility: &'a str,
    pub membership_policy: &'a str,
    pub primary_language_tag: &'a str,
    pub institution_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub discipline_id: Option<Uuid>,
    pub subject_ids: &'a [Uuid],
    pub creator_account_id: Uuid,
}

pub async fn create_community(
    pool: &PgPool,
    new_community: NewCommunity<'_>,
) -> Result<Community, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    validate_directory_references(&mut transaction, &new_community).await?;

    let community = sqlx::query_as::<_, Community>(
        "INSERT INTO communities (
            slug, display_name, description, category, visibility,
            membership_policy, primary_language_tag, institution_id, course_id, discipline_id,
            creator_account_id, owner_account_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
        RETURNING id, slug, display_name, description, category,
            institution_id, course_id, discipline_id, owner_account_id, visibility,
            membership_policy, lifecycle_status, created_at",
    )
    .bind(new_community.slug)
    .bind(new_community.display_name)
    .bind(new_community.description)
    .bind(new_community.category)
    .bind(new_community.visibility)
    .bind(new_community.membership_policy)
    .bind(new_community.primary_language_tag)
    .bind(new_community.institution_id)
    .bind(new_community.course_id)
    .bind(new_community.discipline_id)
    .bind(new_community.creator_account_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(map_community_write_error)?;

    sqlx::query(
        "INSERT INTO community_members (
            community_id, account_id, membership_status, community_role, joined_at
        ) VALUES ($1, $2, 'active', 'owner', now())",
    )
    .bind(community.id)
    .bind(new_community.creator_account_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    record_community_change(
        &mut transaction,
        community.id,
        new_community.creator_account_id,
        None,
        "community_created",
        serde_json::json!({
            "slug": community.slug,
            "category": community.category,
            "visibility": community.visibility
        }),
    )
    .await?;

    for subject_id in new_community.subject_ids {
        sqlx::query(
            "INSERT INTO community_subjects (community_id, subject_id)
             VALUES ($1, $2)",
        )
        .bind(community.id)
        .bind(subject_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    }

    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(community)
}

async fn validate_directory_references(
    transaction: &mut Transaction<'_, Postgres>,
    community: &NewCommunity<'_>,
) -> Result<(), AppError> {
    if !community.subject_ids.is_empty() {
        let available_subject_count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM academic_subjects
             WHERE id = ANY($1) AND is_active = TRUE",
        )
        .bind(community.subject_ids)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
        if available_subject_count != community.subject_ids.len() as i64 {
            return Err(CommunityError::SubjectUnavailable.into());
        }
    }

    if let Some(institution_id) = community.institution_id {
        let institution_is_active = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM institutions WHERE id = $1 AND is_active = TRUE
            )",
        )
        .bind(institution_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
        if !institution_is_active {
            return Err(CommunityError::InstitutionUnavailable.into());
        }
    }

    if let Some(discipline_id) = community.discipline_id {
        let discipline_is_active = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM academic_disciplines WHERE id = $1 AND is_active = TRUE
            )",
        )
        .bind(discipline_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
        if !discipline_is_active {
            return Err(CommunityError::InvalidCategoryReferences.into());
        }
    }

    if let Some(course_id) = community.course_id {
        let course_is_active = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM institution_courses
                WHERE id = $1 AND is_active = TRUE
                    AND ($2::uuid IS NULL OR institution_id = $2)
            )",
        )
        .bind(course_id)
        .bind(community.institution_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| AppError::Internal)?;
        if !course_is_active {
            return Err(CommunityError::CourseUnavailable.into());
        }
    }

    Ok(())
}

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

async fn record_community_change(
    transaction: &mut Transaction<'_, Postgres>,
    community_id: Uuid,
    actor_account_id: Uuid,
    target_account_id: Option<Uuid>,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO community_audit_events (
            community_id, actor_account_id, event_type, target_account_id, payload
        ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(community_id)
    .bind(actor_account_id)
    .bind(event_type)
    .bind(target_account_id)
    .bind(&payload)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    sqlx::query(
        "INSERT INTO outbox_events (event_type, aggregate_type, aggregate_id, payload)
         VALUES ($1, 'community', $2, $3)",
    )
    .bind(event_type)
    .bind(community_id)
    .bind(payload)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok(())
}

fn map_community_write_error(error: sqlx::Error) -> AppError {
    if let Some(database_error) = error.as_database_error()
        && database_error.code().is_some_and(|code| code == "23505")
    {
        return CommunityError::CommunitySlugAlreadyUsed.into();
    }
    AppError::Internal
}
