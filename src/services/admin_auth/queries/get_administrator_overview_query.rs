use crate::{
    errors::app_error::AppError, models::admin_identity::AdministratorOverviewRecord,
    repositories::admin_auth_repository,
};

pub async fn get_administrator_overview(
    pool: &sqlx::PgPool,
) -> Result<AdministratorOverviewRecord, AppError> {
    admin_auth_repository::find_administrator_overview(pool).await
}
