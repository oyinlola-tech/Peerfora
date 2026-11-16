use crate::{
    constants::{
        account_auth::ONE_TIME_TOKEN_QUERY_PARAMETER, agents::AGENT_OPERATOR_CLAIM_PAGE_PATH,
    },
    errors::app_error::AppError,
    integrations::email::templates::agent_operator_claim_email_template::build_agent_operator_claim_email,
    interfaces::email_sender::EmailSender,
};

pub async fn send_agent_operator_claim_email(
    public_url: &str,
    sender: &dyn EmailSender,
    recipient: &str,
    agent_handle: &str,
    token: &str,
) -> Result<(), AppError> {
    let claim_link = format!(
        "{}{page}?{query}={token}",
        public_url.trim_end_matches('/'),
        page = AGENT_OPERATOR_CLAIM_PAGE_PATH,
        query = ONE_TIME_TOKEN_QUERY_PARAMETER,
    );
    let message = build_agent_operator_claim_email(agent_handle, &claim_link);
    sender
        .send_email(recipient, message.subject, &message.plain_text)
        .await
        .map_err(AppError::from)
}
