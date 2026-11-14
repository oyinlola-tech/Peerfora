use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct AdministratorLoginRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub is_active: bool,
    pub mfa_secret_ciphertext: Option<String>,
    pub mfa_locked: bool,
}

#[derive(FromRow)]
pub struct AdministratorChallengeRecord {
    pub administrator_id: Uuid,
    pub email: String,
    pub purpose: String,
    pub encrypted_secret: Option<String>,
    pub is_active: bool,
}

#[derive(sqlx::FromRow)]
pub struct AdministratorMfaSecretRecord {
    pub email: String,
    pub encrypted_secret: String,
}

#[derive(sqlx::FromRow)]
pub struct AdministratorPasswordResetRecipient {
    pub administrator_id: Uuid,
    pub email: String,
}

#[derive(Clone, Copy)]
pub struct AdministratorSessionIdentity {
    pub administrator_id: Uuid,
    pub session_id: Uuid,
}

#[derive(sqlx::FromRow)]
pub struct AdministrativeAuditLogRecord {
    pub id: Uuid,
    pub actor_administrator_id: Uuid,
    pub action: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub result: String,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
pub struct AdministratorSessionRecord {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
pub struct AdministratorOverviewRecord {
    pub registered_accounts: i64,
    pub active_accounts: i64,
    pub pending_email_verification: i64,
    pub suspended_accounts: i64,
    pub total_institutions: i64,
    pub total_communities: i64,
    pub active_communities: i64,
}

#[derive(sqlx::FromRow)]
pub struct AdministrativeAccountRecord {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub account_status: String,
    pub email_verified: bool,
    pub country_code: String,
    pub academic_level_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct AdministrativeAccountFilters<'a> {
    pub search_prefix: Option<&'a str>,
    pub account_status: Option<&'a str>,
    pub country_code: Option<&'a str>,
    pub academic_level_id: Option<&'a str>,
}

#[derive(sqlx::FromRow)]
pub struct AdministrativeCommunityRecord {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub category: String,
    pub visibility: String,
    pub lifecycle_status: String,
    pub owner_account_id: Uuid,
    pub member_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct AdministrativeCommunityFilters<'a> {
    pub search_prefix: Option<&'a str>,
    pub category: Option<&'a str>,
    pub lifecycle_status: Option<&'a str>,
}
