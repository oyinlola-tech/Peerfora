use uuid::Uuid;

use crate::{errors::app_error::AppError, repositories::admin_auth_repository};

pub struct AdministratorSessionProfile {
    pub administrator_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

pub async fn find_administrator_session(
    pool: &sqlx::PgPool,
    administrator_id: Uuid,
) -> Result<AdministratorSessionProfile, AppError> {
    let (email, roles, permissions) =
        admin_auth_repository::find_administrator_session_profile(pool, administrator_id).await?;
    Ok(AdministratorSessionProfile {
        administrator_id,
        email,
        roles,
        permissions,
    })
}
