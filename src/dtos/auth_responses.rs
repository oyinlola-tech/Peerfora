use serde::Serialize;

use crate::models::account::AccountResponse;

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}

#[derive(Serialize)]
pub struct AccountAuthResponse {
    pub account: AccountResponse,
}

impl From<crate::models::account::Account> for AccountAuthResponse {
    fn from(account: crate::models::account::Account) -> Self {
        Self {
            account: account.into(),
        }
    }
}
