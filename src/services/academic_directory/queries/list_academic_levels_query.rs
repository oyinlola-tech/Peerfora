use sqlx::PgPool;

use crate::{
    errors::app_error::AppError, models::academic_directory::AcademicLevel,
    repositories::academic_directory_repository,
};

pub async fn list_academic_levels(pool: &PgPool) -> Result<Vec<AcademicLevel>, AppError> {
    academic_directory_repository::list_active_academic_levels(pool).await
}

pub async fn find_canonical_academic_level_id(
    pool: &PgPool,
    supplied_level_id: &str,
) -> Result<Option<String>, AppError> {
    academic_directory_repository::find_canonical_academic_level_id(pool, supplied_level_id).await
}
