use sqlx::PgPool;

use crate::{errors::app_error::AppError, models::academic_directory::InstitutionDirectoryEntry};

pub async fn search_active_institutions(
    pool: &PgPool,
    search_term: &str,
    result_limit: i64,
) -> Result<Vec<InstitutionDirectoryEntry>, AppError> {
    sqlx::query_as::<_, InstitutionDirectoryEntry>(
        "SELECT institution.id, institution.slug, institution.display_name,
            institution.country_code::text AS country_code, institution.region_name,
            institution.institution_type, institution.official_website
        FROM institutions institution
        WHERE institution.is_active = TRUE
            AND ($1 = '' OR institution.display_name ILIKE '%' || $1 || '%'
                OR institution.slug ILIKE '%' || $1 || '%'
                OR EXISTS (
                    SELECT 1 FROM institution_aliases alias
                    WHERE alias.institution_id = institution.id
                        AND alias.alias ILIKE '%' || $1 || '%'
                ))
        ORDER BY institution.display_name, institution.id
        LIMIT $2",
    )
    .bind(search_term)
    .bind(result_limit)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)
}
