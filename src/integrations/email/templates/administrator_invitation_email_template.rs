use crate::constants::account_auth::ONE_TIME_TOKEN_QUERY_PARAMETER;
use crate::constants::admin_auth::{
    ADMIN_INVITATION_ACCEPT_PAGE_PATH, ADMIN_INVITATION_EMAIL_SUBJECT,
    ADMIN_INVITATION_LIFETIME_HOURS,
};

use super::EmailMessage;

pub fn build_administrator_invitation_email(
    recipient_email: &str,
    role_display_name: &str,
    invitation_token: &str,
    public_url: &str,
) -> EmailMessage {
    let invitation_link = build_administrator_invitation_link(public_url, invitation_token);
    EmailMessage {
        subject: ADMIN_INVITATION_EMAIL_SUBJECT,
        plain_text: format_administrator_invitation_message(
            recipient_email,
            role_display_name,
            &invitation_link,
        ),
    }
}

fn build_administrator_invitation_link(public_url: &str, invitation_token: &str) -> String {
    format!(
        "{}{ADMIN_INVITATION_ACCEPT_PAGE_PATH}?{ONE_TIME_TOKEN_QUERY_PARAMETER}={invitation_token}",
        public_url.trim_end_matches('/')
    )
}

fn format_administrator_invitation_message(
    recipient_email: &str,
    role_display_name: &str,
    invitation_link: &str,
) -> String {
    format!(
        "Hello,\n\nYou have been invited to join the Peerfora administration team as a {}.\n\nThis invitation was sent to {recipient_email}. To accept it, open the one-time link below and choose a secure password:\n{invitation_link}\n\nThe link expires in {ADMIN_INVITATION_LIFETIME_HOURS} hours and can only be used once. After activation, sign in and complete authenticator setup before using administration.\n\nIf you were not expecting this invitation, do not open or forward the link. Contact your Peerfora administrator through a trusted channel.\n\nPeerfora Administration",
        role_display_name,
    )
}
