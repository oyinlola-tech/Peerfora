use crate::{
    constants::admin_auth::{
        ADMIN_ACCOUNTS_DEFAULT_PAGE_SIZE, ADMIN_ACCOUNTS_MAX_OFFSET, ADMIN_ACCOUNTS_MAX_PAGE_SIZE,
    },
    enums::account_lifecycle::AccountLifecycleStatus,
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    models::admin_identity::{AdministrativeAccountFilters, AdministrativeAccountRecord},
    repositories::admin_auth_repository,
};

pub struct AccountAdministrationFilters {
    pub search_prefix: Option<String>,
    pub account_status: Option<String>,
    pub country_code: Option<String>,
    pub academic_level_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn search_accounts_for_administration(
    pool: &sqlx::PgPool,
    requested_filters: AccountAdministrationFilters,
) -> Result<(Vec<AdministrativeAccountRecord>, u32, u32, i64), AppError> {
    let limit = requested_filters
        .limit
        .unwrap_or(ADMIN_ACCOUNTS_DEFAULT_PAGE_SIZE);
    let offset = requested_filters.offset.unwrap_or(0);
    if !(1..=ADMIN_ACCOUNTS_MAX_PAGE_SIZE).contains(&limit) || offset > ADMIN_ACCOUNTS_MAX_OFFSET {
        return Err(AdminAuthError::InvalidPagination.into());
    }
    let search_prefix = normalize_search_prefix(requested_filters.search_prefix)?;
    let account_status = normalize_account_status(requested_filters.account_status)?;
    let country_code = normalize_country_code(requested_filters.country_code)?;
    let academic_level_id = normalize_academic_level(requested_filters.academic_level_id)?;
    let filters = AdministrativeAccountFilters {
        search_prefix: search_prefix.as_deref(),
        account_status: account_status.as_deref(),
        country_code: country_code.as_deref(),
        academic_level_id: academic_level_id.as_deref(),
    };
    let (records, total_count) = admin_auth_repository::search_accounts_for_administration(
        pool,
        &filters,
        i64::from(limit),
        i64::from(offset),
    )
    .await?;
    Ok((records, limit, offset, total_count))
}

pub async fn find_account_for_administration(
    pool: &sqlx::PgPool,
    account_id: uuid::Uuid,
) -> Result<AdministrativeAccountRecord, AppError> {
    admin_auth_repository::find_account_for_administration(pool, account_id)
        .await?
        .ok_or_else(|| AdminAuthError::AccountNotFound.into())
}

fn normalize_search_prefix(search_prefix: Option<String>) -> Result<Option<String>, AppError> {
    let Some(search_prefix) = search_prefix else {
        return Ok(None);
    };
    let search_prefix = search_prefix.trim();
    if search_prefix.is_empty() || search_prefix.len() > 128 {
        return Err(AdminAuthError::InvalidAccountFilter.into());
    }
    Ok(Some(escape_like_prefix(&search_prefix.to_lowercase())))
}

fn escape_like_prefix(search_prefix: &str) -> String {
    search_prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn normalize_account_status(account_status: Option<String>) -> Result<Option<String>, AppError> {
    account_status
        .map(|status| {
            AccountLifecycleStatus::try_from(status.as_str())
                .map(|status| status.as_database_value().to_owned())
                .map_err(|_| AppError::from(AdminAuthError::InvalidAccountFilter))
        })
        .transpose()
}

fn normalize_country_code(country_code: Option<String>) -> Result<Option<String>, AppError> {
    country_code
        .map(|country_code| {
            let normalized = country_code.trim().to_ascii_uppercase();
            if normalized.len() != 2
                || !normalized
                    .bytes()
                    .all(|character| character.is_ascii_uppercase())
            {
                return Err(AdminAuthError::InvalidAccountFilter.into());
            }
            Ok(normalized)
        })
        .transpose()
}

fn normalize_academic_level(academic_level_id: Option<String>) -> Result<Option<String>, AppError> {
    academic_level_id
        .map(|level| {
            let normalized = level.trim().to_lowercase();
            if normalized.is_empty() || normalized.len() > 64 {
                return Err(AdminAuthError::InvalidAccountFilter.into());
            }
            Ok(normalized)
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::escape_like_prefix;

    #[test]
    fn account_prefix_search_treats_like_wildcards_as_literal_characters() {
        assert_eq!(escape_like_prefix(r"a%_b\c"), r"a\%\_b\\c");
    }
}
