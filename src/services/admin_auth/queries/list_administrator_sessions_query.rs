use uuid::Uuid;

use crate::{
    errors::app_error::AppError, models::admin_identity::AdministratorSessionRecord,
    repositories::admin_auth_repository,
};

pub async fn list_administrator_sessions(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
) -> Result<Vec<AdministratorSessionRecord>, AppError> {
    admin_auth_repository::list_administrator_sessions(pool, administrator_id).await
}
