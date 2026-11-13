use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::user_profile::OwnUserProfile,
    repositories::profile_repository,
};

pub async fn find_own_profile(
    pool: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<Option<OwnUserProfile>, AppError> {
    profile_repository::find_own_user_profile(pool, account_id).await
}
