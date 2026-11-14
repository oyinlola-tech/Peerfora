use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct AdministrativeRoleRecord {
    pub id: Uuid,
    pub role_key: String,
    pub display_name: String,
    pub description: String,
    pub is_system_role: bool,
    pub is_active: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, FromRow)]
pub struct AdministrativePermissionRecord {
    pub permission_key: String,
    pub description: String,
    pub is_assignable: bool,
    pub is_sensitive: bool,
}

pub struct NewAdministrativeRole<'a> {
    pub id: Uuid,
    pub role_key: &'a str,
    pub display_name: &'a str,
    pub description: &'a str,
    pub permission_keys: &'a [String],
}
