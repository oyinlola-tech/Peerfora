use crate::{
    errors::app_error::AppError, models::user_profile::PublicUserProfile,
    repositories::profile_repository,
};

pub async fn find_public_profile(
    pool: &sqlx::PgPool,
    username: &str,
) -> Result<Option<PublicUserProfile>, AppError> {
    profile_repository::find_public_user_profile(pool, username).await
}
