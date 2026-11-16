use async_trait::async_trait;

use crate::enums::account_auth_error::AccountAuthError;

pub struct EmailMessage {
    pub subject: &'static str,
    pub plain_text: String,
    pub html: String,
}

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_email(
        &self,
        recipient: &str,
        message: &EmailMessage,
    ) -> Result<(), AccountAuthError>;
}
