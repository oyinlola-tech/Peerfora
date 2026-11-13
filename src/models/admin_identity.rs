use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct AdministratorLoginRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub mfa_secret_ciphertext: Option<String>,
    pub last_totp_step: Option<i64>,
}

#[derive(FromRow)]
pub struct AdministratorChallengeRecord {
    pub administrator_id: Uuid,
    pub email: String,
    pub purpose: String,
    pub encrypted_secret: Option<String>,
    pub is_active: bool,
    pub last_totp_step: Option<i64>,
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
