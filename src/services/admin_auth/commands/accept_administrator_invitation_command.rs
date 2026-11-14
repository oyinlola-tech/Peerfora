use uuid::Uuid;

use crate::{
    constants::admin_auth::{ADMIN_PASSWORD_MAXIMUM_LENGTH, ADMIN_PASSWORD_MINIMUM_LENGTH},
    enums::admin_auth_error::AdminAuthError,
    errors::app_error::AppError,
    repositories::administrator_invitation_repository,
    utils::{one_time_token::hash_one_time_token, password::create_password_hash},
};

pub struct ActivatedAdministrator {
    pub administrator_id: Uuid,
    pub assigned_role: String,
}

pub async fn accept_administrator_invitation(
    pool: &sqlx::PgPool,
    invitation_token: &str,
    password: String,
    correlation_id: Option<&str>,
) -> Result<ActivatedAdministrator, AppError> {
    if !(ADMIN_PASSWORD_MINIMUM_LENGTH..=ADMIN_PASSWORD_MAXIMUM_LENGTH).contains(&password.len()) {
        return Err(AdminAuthError::InvalidAdministratorPasswordLength.into());
    }
    let password_hash = create_password_hash(password).await?;
    let (administrator_id, assigned_role) =
        administrator_invitation_repository::accept_administrator_invitation(
            pool,
            &hash_one_time_token(invitation_token),
            Uuid::new_v4(),
            &password_hash,
            correlation_id,
        )
        .await?;
    Ok(ActivatedAdministrator {
        administrator_id,
        assigned_role,
    })
}
