use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};

use crate::{
    configs::settings::Settings, enums::account_auth_error::AccountAuthError,
    interfaces::email_sender::EmailSender,
};

pub struct SmtpEmailSender {
    host: String,
    port: u16,
    username: String,
    password: String,
    from: Mailbox,
}

impl SmtpEmailSender {
    pub fn from_settings(settings: &Settings) -> Result<Option<Self>, AccountAuthError> {
        let smtp_is_configured = settings.smtp_host.is_some()
            || settings.smtp_username.is_some()
            || settings.smtp_password.is_some()
            || settings.smtp_from.is_some();
        if !smtp_is_configured {
            return Ok(None);
        }
        let Some(host) = settings.smtp_host.clone() else {
            return Err(AccountAuthError::EmailDeliveryNotConfigured);
        };
        let Some(username) = settings.smtp_username.clone() else {
            return Err(AccountAuthError::EmailDeliveryNotConfigured);
        };
        let Some(password) = settings.smtp_password.clone() else {
            return Err(AccountAuthError::EmailDeliveryNotConfigured);
        };
        let Some(from) = settings.smtp_from.as_deref() else {
            return Err(AccountAuthError::EmailDeliveryNotConfigured);
        };
        let from = from
            .parse()
            .map_err(|_| AccountAuthError::EmailDeliveryNotConfigured)?;
        Ok(Some(Self {
            host,
            port: settings.smtp_port,
            username,
            password,
            from,
        }))
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        message: &str,
    ) -> Result<(), AccountAuthError> {
        let recipient: Mailbox = recipient
            .parse()
            .map_err(|_| AccountAuthError::InvalidEmail)?;
        let email = Message::builder()
            .from(self.from.clone())
            .to(recipient)
            .subject(subject)
            .body(message.to_owned())
            .map_err(|_| AccountAuthError::EmailDeliveryNotConfigured)?;
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)
            .map_err(|_| AccountAuthError::EmailDeliveryNotConfigured)?
            .port(self.port)
            .credentials(Credentials::new(
                self.username.clone(),
                self.password.clone(),
            ))
            .build();
        mailer
            .send(email)
            .await
            .map_err(|_| AccountAuthError::EmailDeliveryFailed)?;
        Ok(())
    }
}
