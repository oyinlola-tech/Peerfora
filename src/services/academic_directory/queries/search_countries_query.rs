use sqlx::PgPool;

use crate::{
    constants::academic_directory::{
        DEFAULT_DIRECTORY_PAGE_SIZE, MAX_DIRECTORY_PAGE_SIZE, MAX_DIRECTORY_SEARCH_LENGTH,
    },
    enums::directory_error::DirectoryError,
    errors::app_error::AppError,
    models::academic_directory::Country,
    repositories::academic_directory_repository,
};

pub async fn search_countries(
    pool: &PgPool,
    search_term: Option<&str>,
    requested_limit: Option<i64>,
) -> Result<Vec<Country>, AppError> {
    let search_term = search_term.unwrap_or_default().trim();
    if search_term.chars().count() > MAX_DIRECTORY_SEARCH_LENGTH {
        return Err(DirectoryError::SearchTermTooLong.into());
    }
    let limit = requested_limit
        .unwrap_or(DEFAULT_DIRECTORY_PAGE_SIZE)
        .clamp(1, MAX_DIRECTORY_PAGE_SIZE);
    academic_directory_repository::search_countries(pool, search_term, limit).await
}
