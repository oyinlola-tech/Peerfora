use std::sync::OnceLock;

use minijinja::{Environment, UndefinedBehavior, context, value::Value};

use crate::{
    constants::{account_auth, admin_auth, agents},
    errors::app_error::AppError,
    interfaces::email_sender::EmailMessage,
};

use super::theme::provisional_theme;

pub enum TransactionalEmail<'a> {
    VerifyEmail {
        link: &'a str,
    },
    ResetPassword {
        link: &'a str,
    },
    AdministratorInvitation {
        link: &'a str,
        recipient: &'a str,
        role: &'a str,
    },
    AdministratorReset {
        link: &'a str,
    },
    AgentClaim {
        link: &'a str,
        handle: &'a str,
    },
}

static ENVIRONMENT: OnceLock<Result<Environment<'static>, ()>> = OnceLock::new();

fn environment() -> Result<&'static Environment<'static>, AppError> {
    ENVIRONMENT
        .get_or_init(|| {
            let mut env = Environment::new();
            env.set_undefined_behavior(UndefinedBehavior::Strict);
            for (name, source) in [
                (
                    "layouts/base.html",
                    include_str!("../../../../templates/emails/layouts/base.html"),
                ),
                (
                    "partials/header.html",
                    include_str!("../../../../templates/emails/partials/header.html"),
                ),
                (
                    "partials/footer.html",
                    include_str!("../../../../templates/emails/partials/footer.html"),
                ),
                (
                    "partials/button.html",
                    include_str!("../../../../templates/emails/partials/button.html"),
                ),
                (
                    "auth/verify_email.html",
                    include_str!("../../../../templates/emails/auth/verify_email.html"),
                ),
                (
                    "auth/verify_email.txt",
                    include_str!("../../../../templates/emails/auth/verify_email.txt"),
                ),
                (
                    "auth/reset_password.html",
                    include_str!("../../../../templates/emails/auth/reset_password.html"),
                ),
                (
                    "auth/reset_password.txt",
                    include_str!("../../../../templates/emails/auth/reset_password.txt"),
                ),
                (
                    "administration/invitation.html",
                    include_str!("../../../../templates/emails/administration/invitation.html"),
                ),
                (
                    "administration/invitation.txt",
                    include_str!("../../../../templates/emails/administration/invitation.txt"),
                ),
                (
                    "administration/reset_password.html",
                    include_str!("../../../../templates/emails/administration/reset_password.html"),
                ),
                (
                    "administration/reset_password.txt",
                    include_str!("../../../../templates/emails/administration/reset_password.txt"),
                ),
                (
                    "agents/claim.html",
                    include_str!("../../../../templates/emails/agents/claim.html"),
                ),
                (
                    "agents/claim.txt",
                    include_str!("../../../../templates/emails/agents/claim.txt"),
                ),
            ] {
                env.add_template(name, source).map_err(|_| ())?;
            }
            Ok(env)
        })
        .as_ref()
        .map_err(|_| AppError::Internal)
}

pub fn render_email(
    template: TransactionalEmail<'_>,
    _requested_locale: Option<&str>,
) -> Result<EmailMessage, AppError> {
    let (path, subject, link, detail, recipient, expiry, action_label) = match template {
        TransactionalEmail::VerifyEmail { link } => (
            "auth/verify_email",
            account_auth::EMAIL_VERIFICATION_SUBJECT,
            link,
            "",
            "",
            account_auth::EMAIL_VERIFICATION_LIFETIME_HOURS,
            "Confirm email",
        ),
        TransactionalEmail::ResetPassword { link } => (
            "auth/reset_password",
            account_auth::PASSWORD_RESET_SUBJECT,
            link,
            "",
            "",
            account_auth::PASSWORD_RESET_LIFETIME_HOURS,
            "Reset password",
        ),
        TransactionalEmail::AdministratorInvitation {
            link,
            recipient,
            role,
        } => (
            "administration/invitation",
            admin_auth::ADMIN_INVITATION_EMAIL_SUBJECT,
            link,
            role,
            recipient,
            admin_auth::ADMIN_INVITATION_LIFETIME_HOURS,
            "Accept invitation",
        ),
        TransactionalEmail::AdministratorReset { link } => (
            "administration/reset_password",
            admin_auth::ADMIN_PASSWORD_RESET_EMAIL_SUBJECT,
            link,
            "",
            "",
            admin_auth::ADMIN_PASSWORD_RESET_LIFETIME_MINUTES,
            "Reset password",
        ),
        TransactionalEmail::AgentClaim { link, handle } => (
            "agents/claim",
            agents::AGENT_OPERATOR_CLAIM_SUBJECT,
            link,
            handle,
            "",
            i64::from(agents::AGENT_OPERATOR_CLAIM_LIFETIME_MINUTES),
            "Claim agent",
        ),
    };
    let theme = provisional_theme()?;
    let ctx: Value = context!(
        locale => "en",
        brand_name => theme.brand_name.as_str(),
        body_background => theme.body_background.as_str(),
        content_surface => theme.content_surface.as_str(),
        primary_text => theme.primary_text.as_str(),
        secondary_text => theme.secondary_text.as_str(),
        border => theme.border.as_str(),
        action_background => theme.action_background.as_str(),
        action_text => theme.action_text.as_str(),
        link_color => theme.link.as_str(),
        body_font => theme.body_font.as_str(),
        content_width_px => theme.content_width_px,
        link => link,
        detail => detail,
        recipient => recipient,
        expiry => expiry,
        action_label => action_label,
    );
    let env = environment()?;
    let html = env
        .get_template(&format!("{path}.html"))
        .and_then(|template| template.render(ctx.clone()))
        .map_err(|_| AppError::Internal)?;
    let plain_text = env
        .get_template(&format!("{path}.txt"))
        .and_then(|template| template.render(ctx))
        .map_err(|_| AppError::Internal)?;
    Ok(EmailMessage {
        subject: subject.to_owned(),
        plain_text,
        html,
    })
}

#[cfg(test)]
mod tests {
    use super::{TransactionalEmail, render_email};

    #[test]
    fn templates_escape_html_and_include_plain_text_and_fallback_locale() {
        let message = render_email(
            TransactionalEmail::AgentClaim {
                link: "https://example.test/claim?token=123",
                handle: "<script>alert('x')</script>",
            },
            Some("fr"),
        )
        .unwrap();
        assert!(message.html.contains("&lt;script&gt;"));
        assert!(!message.html.contains("<script>"));
        assert!(message.html.contains("lang=\"en\""));
        assert!(message.plain_text.contains("<script>alert('x')</script>"));
        assert!(
            message
                .plain_text
                .contains("https://example.test/claim?token=123")
        );
        assert!(message.plain_text.contains("60 minutes"));
    }

    #[test]
    fn invitation_fields_and_url_are_escaped_in_html() {
        let message = render_email(
            TransactionalEmail::AdministratorInvitation {
                link: "https://example.test/path?token=abc&next=\"quoted\"",
                recipient: "<someone>@example.test",
                role: "<admin>",
            },
            None,
        )
        .unwrap();
        assert!(!message.html.contains("<admin>"));
        assert!(!message.html.contains("<someone>"));
        assert!(message.html.contains("&lt;admin&gt;"));
        assert!(message.html.contains("&lt;someone&gt;"));
        assert!(!message.html.contains("&next=\"quoted\""));
        assert!(message.plain_text.contains("&next=\"quoted\""));
    }

    #[test]
    fn all_active_templates_render_html_and_plain_text() {
        let link = "https://example.test/path?token=123";
        let templates = [
            TransactionalEmail::VerifyEmail { link },
            TransactionalEmail::ResetPassword { link },
            TransactionalEmail::AdministratorInvitation {
                link,
                recipient: "user@example.test",
                role: "Support Agent",
            },
            TransactionalEmail::AdministratorReset { link },
            TransactionalEmail::AgentClaim {
                link,
                handle: "agent",
            },
        ];
        for template in templates {
            let message = render_email(template, None).unwrap();
            assert!(message.html.contains("<!doctype html>"));
            assert!(message.html.contains("example.test"));
            assert!(message.plain_text.contains(link));
            assert!(!message.subject.is_empty());
        }
    }
}
