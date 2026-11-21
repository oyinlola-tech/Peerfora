//! In-memory transport for unit tests only; it never sends mail or logs message bodies.
use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    enums::account_auth_error::AccountAuthError,
    interfaces::email_sender::{EmailMessage, EmailSender},
};

#[derive(Default)]
struct TestSender {
    messages: Mutex<Vec<(String, EmailMessage)>>,
}

#[async_trait]
impl EmailSender for TestSender {
    async fn send_email(
        &self,
        recipient: &str,
        message: &EmailMessage,
    ) -> Result<(), AccountAuthError> {
        self.messages
            .lock()
            .map_err(|_| AccountAuthError::EmailDeliveryFailed)?
            .push((
                recipient.to_owned(),
                EmailMessage {
                    subject: message.subject.clone(),
                    plain_text: message.plain_text.clone(),
                    html: message.html.clone(),
                },
            ));
        Ok(())
    }
}

#[tokio::test]
async fn records_both_alternatives_without_sending() {
    use super::templates::transactional_email::{TransactionalEmail, render_email};
    let sender = TestSender::default();
    let message = render_email(
        TransactionalEmail::VerifyEmail {
            link: "https://example.test/verify?token=example",
        },
        None,
    )
    .unwrap();
    sender
        .send_email("person@example.test", &message)
        .await
        .unwrap();
    let messages = sender.messages.lock().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].0, "person@example.test");
    assert!(messages[0].1.html.contains("example.test"));
    assert!(messages[0].1.plain_text.contains("example.test"));
}
