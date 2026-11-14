pub struct EmailMessage {
    pub subject: &'static str,
    pub plain_text: String,
}

pub mod account_access_email_templates;
pub mod administrator_invitation_email_template;
pub mod administrator_password_reset_email_template;
