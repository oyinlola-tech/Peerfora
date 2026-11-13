use sqlx::PgPool;

use crate::{
    enums::directory_error::DirectoryError, errors::app_error::AppError,
    models::academic_directory::AcademicSubject, repositories::academic_directory_repository,
};

pub async fn search_academic_subjects(
    pool: &PgPool,
    search_term: Option<&str>,
    requested_limit: Option<i64>,
) -> Result<Vec<AcademicSubject>, AppError> {
    let search_term = search_term.unwrap_or_default().trim();
    if search_term.chars().count() > 100 {
        return Err(DirectoryError::SearchTermTooLong.into());
    }

    let result_limit = requested_limit.unwrap_or(20).clamp(1, 50);
    academic_directory_repository::search_academic_subjects(pool, search_term, result_limit).await
}
