use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    enums::community_error::CommunityError, errors::app_error::AppError,
    models::community::Community,
};

use super::audit::{map_community_write_error, record_community_change};

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
