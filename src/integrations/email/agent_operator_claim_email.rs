use crate::{
    errors::app_error::AppError,
    integrations::email::{
        links::{EmailLink, build_email_link},
        templates::transactional_email::{TransactionalEmail, render_email},
    },
    interfaces::email_sender::EmailSender,
};

pub async fn send_agent_operator_claim_email(
    public_url: &str,
    sender: &dyn EmailSender,
    recipient: &str,
    agent_handle: &str,
    token: &str,
) -> Result<(), AppError> {
    let link = build_email_link(public_url, EmailLink::AgentClaim, token)?;
    let message = render_email(
        TransactionalEmail::AgentClaim {
            link: &link,
            handle: agent_handle,
        },
        None,
    )?;
    sender
        .send_email(recipient, &message)
        .await
        .map_err(AppError::from)
}
