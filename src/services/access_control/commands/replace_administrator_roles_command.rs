use uuid::Uuid;

use crate::{
    errors::app_error::AppError, repositories::admin_role_assignment_repository,
    services::access_control::validation::validate_role_ids,
};

pub async fn replace_administrator_roles(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    target_administrator_id: Uuid,
    role_ids: &[Uuid],
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    validate_role_ids(role_ids)?;
    admin_role_assignment_repository::replace_administrator_roles(
        pool,
        actor_id,
        target_administrator_id,
        role_ids,
        correlation_id,
    )
    .await
}
