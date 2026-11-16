//! Offline preview with fixed example data only; no production tokens or email addresses.
use peerfora_api::integrations::email::templates::transactional_email::{
    TransactionalEmail, render_email,
};

fn main() {
    if let Err(error) = preview() {
        eprintln!("Email preview failed: {error}");
        std::process::exit(1);
    }
}

fn preview() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let name = args
        .next()
        .ok_or("expected: verify|reset|admin-invitation|admin-reset|agent-claim [html|text]")?;
    let format = args.next().unwrap_or_else(|| "html".to_owned());
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let link = "https://example.test/example?token=example-only";
    let template = match name.as_str() {
        "verify" => TransactionalEmail::VerifyEmail { link },
        "reset" => TransactionalEmail::ResetPassword { link },
        "admin-invitation" => TransactionalEmail::AdministratorInvitation {
            link,
            recipient: "person@example.test",
            role: "Support Agent",
        },
        "admin-reset" => TransactionalEmail::AdministratorReset { link },
        "agent-claim" => TransactionalEmail::AgentClaim {
            link,
            handle: "example-agent",
        },
        _ => return Err("unknown template".into()),
    };
    let message = render_email(template, None).map_err(|_| "could not render preview")?;
    match format.as_str() {
        "html" => print!("{}", message.html),
        "text" => print!("{}", message.plain_text),
        _ => return Err("format must be html or text".into()),
    }
    Ok(())
}
