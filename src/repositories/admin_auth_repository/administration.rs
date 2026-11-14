use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    models::admin_identity::{
        AdministrativeAccountFilters, AdministrativeAccountRecord, AdministratorOverviewRecord,
    },
};

pub async fn find_administrator_overview(
    pool: &PgPool,
) -> Result<AdministratorOverviewRecord, AppError> {
    sqlx::query_as::<_, AdministratorOverviewRecord>(
        "SELECT count(*) AS registered_accounts,
            count(*) FILTER (WHERE account_status = 'active') AS active_accounts,
            count(*) FILTER (WHERE account_status = 'active' AND email_verified_at IS NULL)
                AS pending_email_verification,
            count(*) FILTER (WHERE account_status = 'suspended') AS suspended_accounts,
            (SELECT count(*) FROM institutions) AS total_institutions,
            (SELECT count(*) FROM communities) AS total_communities,
            (SELECT count(*) FROM communities WHERE lifecycle_status = 'active')
                AS active_communities
        FROM accounts",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn search_accounts_for_administration(
    pool: &PgPool,
    filters: &AdministrativeAccountFilters<'_>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AdministrativeAccountRecord>, i64), AppError> {
    let total_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
        FROM accounts account
        LEFT JOIN user_profiles profile ON profile.account_id = account.id
        WHERE ($1::text IS NULL OR lower(account.email) LIKE $1 || '%' ESCAPE E'\\\\'
                OR lower(account.username) LIKE $1 || '%' ESCAPE E'\\\\')
            AND ($2::text IS NULL OR account.account_status = $2)
            AND ($3::text IS NULL OR account.eligibility_country_code = $3)
            AND ($4::text IS NULL OR profile.academic_level_id = $4)",
    )
    .bind(filters.search_prefix)
    .bind(filters.account_status)
    .bind(filters.country_code)
    .bind(filters.academic_level_id)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    let records = sqlx::query_as::<_, AdministrativeAccountRecord>(
        "SELECT account.id, account.email, account.username,
            account.account_status, account.email_verified_at IS NOT NULL AS email_verified,
            account.eligibility_country_code::text AS country_code,
            profile.academic_level_id, account.created_at
        FROM accounts account
        LEFT JOIN user_profiles profile ON profile.account_id = account.id
        WHERE ($1::text IS NULL OR lower(account.email) LIKE $1 || '%' ESCAPE E'\\\\'
                OR lower(account.username) LIKE $1 || '%' ESCAPE E'\\\\')
            AND ($2::text IS NULL OR account.account_status = $2)
            AND ($3::text IS NULL OR account.eligibility_country_code = $3)
            AND ($4::text IS NULL OR profile.academic_level_id = $4)
        ORDER BY account.created_at DESC, account.id
        LIMIT $5 OFFSET $6",
    )
    .bind(filters.search_prefix)
    .bind(filters.account_status)
    .bind(filters.country_code)
    .bind(filters.academic_level_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    Ok((records, total_count))
}

pub async fn find_account_for_administration(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<AdministrativeAccountRecord>, AppError> {
    sqlx::query_as::<_, AdministrativeAccountRecord>(
        "SELECT account.id, account.email, account.username,
            account.account_status, account.email_verified_at IS NOT NULL AS email_verified,
            account.eligibility_country_code::text AS country_code,
            profile.academic_level_id, account.created_at
        FROM accounts account
        LEFT JOIN user_profiles profile ON profile.account_id = account.id
        WHERE account.id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}
