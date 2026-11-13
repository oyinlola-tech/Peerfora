use sqlx::PgPool;

use crate::{
    errors::app_error::AppError,
    models::academic_directory::{AcademicLevel, AcademicSubject, Country},
};

pub async fn search_countries(
    pool: &PgPool,
    search_term: &str,
    result_limit: i64,
) -> Result<Vec<Country>, AppError> {
    sqlx::query_as::<_, Country>(
        "SELECT country_code::text, alpha_3::text, display_name, official_name
        FROM countries
        WHERE is_active = TRUE
            AND ($1 = '' OR country_code ILIKE $1 || '%'
                OR alpha_3 ILIKE $1 || '%'
                OR display_name ILIKE '%' || $1 || '%'
                OR official_name ILIKE '%' || $1 || '%'
                OR EXISTS (
                    SELECT 1 FROM country_name_aliases alias
                    WHERE alias.country_code = countries.country_code
                        AND alias.display_name ILIKE '%' || $1 || '%'
                ))
        ORDER BY display_name, country_code
        LIMIT $2",
    )
    .bind(search_term)
    .bind(result_limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn list_active_academic_levels(pool: &PgPool) -> Result<Vec<AcademicLevel>, AppError> {
    sqlx::query_as::<_, AcademicLevel>(
        "SELECT level.id, level.display_name, level.display_order,
            COALESCE(array_agg(alias.alias ORDER BY alias.alias)
                FILTER (WHERE alias.alias IS NOT NULL), ARRAY[]::text[]) AS aliases
        FROM academic_levels level
        LEFT JOIN academic_level_aliases alias ON alias.academic_level_id = level.id
        WHERE level.is_active = TRUE
        GROUP BY level.id
        ORDER BY level.display_order, level.id",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn find_canonical_academic_level_id(
    pool: &PgPool,
    supplied_level_id: &str,
) -> Result<Option<String>, AppError> {
    sqlx::query_scalar(
        "SELECT level.id FROM academic_levels level
        LEFT JOIN academic_level_aliases alias
            ON alias.academic_level_id = level.id AND lower(alias.alias) = lower($1)
        WHERE level.is_active = TRUE
            AND (lower(level.id) = lower($1) OR alias.id IS NOT NULL)
        LIMIT 1",
    )
    .bind(supplied_level_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn search_academic_subjects(
    pool: &PgPool,
    search_term: &str,
    result_limit: i64,
) -> Result<Vec<AcademicSubject>, AppError> {
    sqlx::query_as::<_, AcademicSubject>(
        "SELECT subject.id, subject.slug, subject.display_name, subject.description,
            discipline.display_name AS discipline_name
        FROM academic_subjects subject
        LEFT JOIN academic_disciplines discipline ON discipline.id = subject.discipline_id
        WHERE subject.is_active = TRUE
            AND ($1 = '' OR subject.display_name ILIKE '%' || $1 || '%'
                OR subject.slug ILIKE '%' || $1 || '%'
                OR EXISTS (
                    SELECT 1 FROM academic_subject_aliases alias
                    WHERE alias.subject_id = subject.id
                        AND alias.alias ILIKE '%' || $1 || '%'
                ))
        ORDER BY subject.display_name, subject.id
        LIMIT $2",
    )
    .bind(search_term)
    .bind(result_limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
