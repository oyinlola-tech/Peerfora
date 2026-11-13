use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::user_profile::AccountPreferences,
    repositories::profile_repository,
};

pub async fn find_account_preferences(
    pool: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<Option<AccountPreferences>, AppError> {
    profile_repository::find_account_preferences(pool, account_id).await
}
