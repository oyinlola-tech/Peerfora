use crate::{
    constants::admin_auth::{
        ADMIN_COMMUNITIES_DEFAULT_PAGE_SIZE, ADMIN_COMMUNITIES_MAX_OFFSET,
        ADMIN_COMMUNITIES_MAX_PAGE_SIZE,
    },
    enums::{
        admin_auth_error::AdminAuthError,
        community::{CommunityCategory, CommunityLifecycleStatus},
    },
    errors::app_error::AppError,
    models::admin_identity::{AdministrativeCommunityFilters, AdministrativeCommunityRecord},
    repositories::community_administration_repository,
};

pub struct CommunityAdministrationFilters {
    pub search_prefix: Option<String>,
    pub category: Option<String>,
    pub lifecycle_status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn search_communities_for_administration(
    pool: &sqlx::PgPool,
    requested_filters: CommunityAdministrationFilters,
) -> Result<(Vec<AdministrativeCommunityRecord>, u32, u32, i64), AppError> {
    let limit = requested_filters
        .limit
        .unwrap_or(ADMIN_COMMUNITIES_DEFAULT_PAGE_SIZE);
    let offset = requested_filters.offset.unwrap_or(0);
    if !(1..=ADMIN_COMMUNITIES_MAX_PAGE_SIZE).contains(&limit)
        || offset > ADMIN_COMMUNITIES_MAX_OFFSET
    {
        return Err(AdminAuthError::InvalidPagination.into());
    }
    let search_prefix = normalize_community_search_prefix(requested_filters.search_prefix)?;
    let category = normalize_community_category(requested_filters.category)?;
    let lifecycle_status =
        normalize_community_lifecycle_status(requested_filters.lifecycle_status)?;
    let filters = AdministrativeCommunityFilters {
        search_prefix: search_prefix.as_deref(),
        category: category.as_deref(),
        lifecycle_status: lifecycle_status.as_deref(),
    };
    let (records, total_count) =
        community_administration_repository::search_communities_for_administration(
            pool,
            &filters,
            i64::from(limit),
            i64::from(offset),
        )
        .await?;
    Ok((records, limit, offset, total_count))
}

pub async fn find_community_for_administration(
    pool: &sqlx::PgPool,
    community_id: uuid::Uuid,
) -> Result<AdministrativeCommunityRecord, AppError> {
    community_administration_repository::find_community_for_administration(pool, community_id)
        .await?
        .ok_or_else(|| AdminAuthError::CommunityNotFound.into())
}

fn normalize_community_search_prefix(
    search_prefix: Option<String>,
) -> Result<Option<String>, AppError> {
    let Some(search_prefix) = search_prefix else {
        return Ok(None);
    };
    let search_prefix = search_prefix.trim().to_lowercase();
    if search_prefix.is_empty() || search_prefix.len() > 128 {
        return Err(AdminAuthError::InvalidCommunityFilter.into());
    }
    Ok(Some(escape_like_prefix(&search_prefix)))
}

fn normalize_community_category(category: Option<String>) -> Result<Option<String>, AppError> {
    category
        .map(|category| {
            let category = category.trim().to_lowercase();
            let category = CommunityCategory::try_from(category.as_str())
                .map_err(|_| AdminAuthError::InvalidCommunityFilter)?;
            Ok::<String, AdminAuthError>(category.as_database_value().to_owned())
        })
        .transpose()
        .map_err(AppError::from)
}

fn normalize_community_lifecycle_status(
    lifecycle_status: Option<String>,
) -> Result<Option<String>, AppError> {
    lifecycle_status
        .map(|status| {
            CommunityLifecycleStatus::try_from(status.trim())
                .map(|status| status.as_database_value().to_owned())
        })
        .transpose()
        .map_err(AppError::from)
}

fn escape_like_prefix(search_prefix: &str) -> String {
    search_prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
