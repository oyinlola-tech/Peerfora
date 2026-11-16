use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(FromRow)]
pub struct DeveloperProfileRecord {
    pub account_id: Uuid,
    pub display_name: String,
    pub description: String,
    pub profile_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperApplicationScopeRecord {
    pub scope_key: String,
    pub scope_status: String,
}

#[derive(FromRow)]
pub struct DeveloperApplicationRecord {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub application_type: String,
    pub application_status: String,
    pub scopes: Json<Vec<DeveloperApplicationScopeRecord>>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct AdministrativeDeveloperApplicationRecord {
    pub id: Uuid,
    pub owner_account_id: Uuid,
    pub owner_username: String,
    pub name: String,
    pub description: String,
    pub application_type: String,
    pub application_status: String,
    pub scopes: Json<Vec<DeveloperApplicationScopeRecord>>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct DeveloperCredentialRecord {
    pub id: Uuid,
    pub application_id: Uuid,
    pub name: String,
    pub environment: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct DeveloperCredentialIdentity {
    pub credential_id: Uuid,
    pub application_id: Uuid,
}
