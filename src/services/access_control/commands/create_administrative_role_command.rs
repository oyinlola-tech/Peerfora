use uuid::Uuid;

use crate::{
    errors::app_error::AppError,
    models::admin_role::NewAdministrativeRole,
    repositories::admin_role_repository,
    services::access_control::validation::{
        validate_permission_keys, validate_role_key, validate_role_text,
    },
};

pub async fn create_administrative_role(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    role_key: &str,
    display_name: &str,
    description: &str,
    permission_keys: &[String],
    correlation_id: Option<&str>,
) -> Result<Uuid, AppError> {
    validate_role_key(role_key)?;
    validate_role_text(display_name, description)?;
    validate_permission_keys(permission_keys)?;
    let role_id = Uuid::new_v4();
    admin_role_repository::create_administrative_role(
        pool,
        actor_id,
        NewAdministrativeRole {
            id: role_id,
            role_key,
            display_name: display_name.trim(),
            description: description.trim(),
            permission_keys,
        },
        correlation_id,
    )
    .await?;
    Ok(role_id)
}
