use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAdministrativeRoleRequest {
    pub role_key: String,
    pub display_name: String,
    pub description: String,
    pub permission_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAdministrativeRoleRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceAdministrativeRolePermissionsRequest {
    pub permission_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceAdministratorRolesRequest {
    pub role_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeRolesResponse {
    pub items: Vec<AdministrativeRoleItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativeRoleItem {
    pub id: Uuid,
    pub role_key: String,
    pub display_name: String,
    pub description: String,
    pub is_system_role: bool,
    pub is_active: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativePermissionsResponse {
    pub items: Vec<AdministrativePermissionItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministrativePermissionItem {
    pub permission_key: String,
    pub description: String,
    pub is_assignable: bool,
    pub is_sensitive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorPermissionKeysResponse {
    pub administrator_id: Uuid,
    pub permission_keys: Vec<String>,
}
