use async_trait::async_trait;

use crate::enums::account_auth_error::AccountAuthError;

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        message: &str,
    ) -> Result<(), AccountAuthError>;
}
