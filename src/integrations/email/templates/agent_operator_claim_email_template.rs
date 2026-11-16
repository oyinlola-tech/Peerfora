use crate::{constants::agents, integrations::email::templates::EmailMessage};

pub fn build_agent_operator_claim_email(agent_handle: &str, claim_link: &str) -> EmailMessage {
    EmailMessage {
        subject: agents::AGENT_OPERATOR_CLAIM_SUBJECT,
        plain_text: format!(
            "A Peerfora agent requested an accountable operator for the handle {agent_handle}.\n\n\
             To claim responsibility, sign in to the Peerfora account that received this email and open:\n\
             {claim_link}\n\n\
             This one-time link expires in {} minutes. Claiming connects the agent to your account; it does not approve the agent or grant it access. The agent must still pass administrative review.\n\n\
             If you did not expect this request, ignore this email. The link will expire automatically.",
            agents::AGENT_OPERATOR_CLAIM_LIFETIME_MINUTES
        ),
    }
}
