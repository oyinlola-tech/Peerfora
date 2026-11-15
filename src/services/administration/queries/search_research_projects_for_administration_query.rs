use uuid::Uuid;

use crate::{
    constants::admin_auth::{
        ADMIN_RESEARCH_PROJECTS_DEFAULT_PAGE_SIZE, ADMIN_RESEARCH_PROJECTS_MAX_OFFSET,
        ADMIN_RESEARCH_PROJECTS_MAX_PAGE_SIZE,
    },
    enums::{admin_auth_error::AdminAuthError, research::ResearchProjectStatus},
    errors::app_error::AppError,
    models::admin_research::{
        AdministrativeResearchProjectFilters, AdministrativeResearchProjectRecord,
    },
    repositories::research_administration_repository,
};

pub struct ResearchProjectAdministrationFilters {
    pub search_prefix: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn search_research_projects_for_administration(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
    filters: ResearchProjectAdministrationFilters,
    correlation_id: Option<&str>,
) -> Result<(Vec<AdministrativeResearchProjectRecord>, u32, u32, i64), AppError> {
    let limit = filters
        .limit
        .unwrap_or(ADMIN_RESEARCH_PROJECTS_DEFAULT_PAGE_SIZE);
    let offset = filters.offset.unwrap_or(0);
    if !(1..=ADMIN_RESEARCH_PROJECTS_MAX_PAGE_SIZE).contains(&limit)
        || offset > ADMIN_RESEARCH_PROJECTS_MAX_OFFSET
    {
        return Err(AdminAuthError::InvalidPagination.into());
    }
    let search_prefix = normalize_search_prefix(filters.search_prefix)?;
    let project_status = normalize_project_status(filters.status)?;
    let filters = AdministrativeResearchProjectFilters {
        search_prefix: search_prefix.as_deref(),
        project_status,
    };
    let (records, total_count) =
        research_administration_repository::search_research_projects_for_administration(
            pool,
            filters,
            administrator_id,
            i64::from(limit),
            i64::from(offset),
            correlation_id,
        )
        .await?;
    Ok((records, limit, offset, total_count))
}

fn normalize_search_prefix(search_prefix: Option<String>) -> Result<Option<String>, AppError> {
    let Some(search_prefix) = search_prefix else {
        return Ok(None);
    };
    let search_prefix = search_prefix.trim().to_lowercase();
    if search_prefix.is_empty() || search_prefix.chars().count() > 128 {
        return Err(AdminAuthError::InvalidResearchProjectFilter.into());
    }
    Ok(Some(escape_like_prefix(&search_prefix)))
}

fn normalize_project_status(
    status: Option<String>,
) -> Result<Option<ResearchProjectStatus>, AppError> {
    status
        .map(|status| {
            ResearchProjectStatus::from_database_value(status.trim())
                .ok_or(AdminAuthError::InvalidResearchProjectFilter)
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
