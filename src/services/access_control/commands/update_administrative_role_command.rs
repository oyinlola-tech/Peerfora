use uuid::Uuid;

use crate::{
    enums::admin_auth_error::AdminAuthError, errors::app_error::AppError,
    repositories::admin_role_repository, services::access_control::validation::validate_role_text,
};

pub async fn update_administrative_role(
    pool: &sqlx::PgPool,
    actor_id: Uuid,
    role_id: Uuid,
    display_name: Option<String>,
    description: Option<String>,
    is_active: Option<bool>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    if display_name.is_none() && description.is_none() && is_active.is_none() {
        return Err(AdminAuthError::EmptyRoleUpdate.into());
    }
    if let (Some(display_name), Some(description)) = (&display_name, &description) {
        validate_role_text(display_name, description)?;
    }
    if let Some(display_name) = display_name.as_deref()
        && !(3..=80).contains(&display_name.trim().chars().count())
    {
        return Err(AdminAuthError::InvalidRoleDetails.into());
    }
    if let Some(description) = description.as_deref()
        && (description.trim().is_empty() || description.chars().count() > 500)
    {
        return Err(AdminAuthError::InvalidRoleDetails.into());
    }
    admin_role_repository::update_administrative_role(
        pool,
        actor_id,
        role_id,
        display_name.as_deref().map(str::trim),
        description.as_deref().map(str::trim),
        is_active,
        correlation_id,
    )
    .await
}
