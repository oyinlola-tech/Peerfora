use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub password_hash: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email_verified: bool,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            email: account.email,
            username: account.username,
            display_name: account.display_name,
            email_verified: account.email_verified_at.is_some(),
        }
    }
}
