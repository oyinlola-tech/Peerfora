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
