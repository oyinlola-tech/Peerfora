use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};

use crate::{
    configs::settings::Settings,
    enums::account_auth_error::AccountAuthError,
    interfaces::email_sender::{EmailMessage, EmailSender},
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
        message: &EmailMessage,
    ) -> Result<(), AccountAuthError> {
        let email = build_smtp_message(&self.from, recipient, message)?;
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

fn build_smtp_message(
    from: &Mailbox,
    recipient: &str,
    message: &EmailMessage,
) -> Result<Message, AccountAuthError> {
    let recipient: Mailbox = recipient
        .parse()
        .map_err(|_| AccountAuthError::InvalidEmail)?;
    Message::builder()
        .from(from.clone())
        .to(recipient)
        .subject(message.subject.clone())
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(message.plain_text.clone()))
                .singlepart(SinglePart::html(message.html.clone())),
        )
        .map_err(|_| AccountAuthError::EmailDeliveryNotConfigured)
}

#[cfg(test)]
mod tests {
    use super::build_smtp_message;
    use crate::interfaces::email_sender::EmailMessage;

    #[test]
    fn smtp_message_contains_both_alternative_parts() {
        let from = "sender@example.test".parse().unwrap();
        let message = EmailMessage {
            subject: "Example subject".to_owned(),
            plain_text: "plain example".to_owned(),
            html: "<p>html example</p>".to_owned(),
        };
        let email = build_smtp_message(&from, "recipient@example.test", &message).unwrap();
        let wire = String::from_utf8(email.formatted()).unwrap();
        assert!(wire.contains("multipart/alternative"));
        assert!(wire.contains("text/plain"));
        assert!(wire.contains("text/html"));
        assert!(wire.contains("plain example"));
        assert!(wire.contains("html example"));
        assert!(
            build_smtp_message(
                &from,
                "invalid
address",
                &message
            )
            .is_err()
        );
    }
}
