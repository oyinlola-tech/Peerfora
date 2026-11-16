use url::Url;

use crate::{
    configs::url_settings::validate_public_url,
    constants::{account_auth, admin_auth, agents},
    errors::app_error::AppError,
    validations::account_auth_validation::validate_verification_token,
};

pub enum EmailLink {
    VerifyEmail,
    ResetPassword,
    AdministratorInvitation,
    AdministratorReset,
    AgentClaim,
}

pub fn build_email_link(
    origin: &str,
    destination: EmailLink,
    token: &str,
) -> Result<String, AppError> {
    validate_public_url(origin, false).map_err(|_| AppError::Internal)?;
    validate_verification_token(token).map_err(|_| AppError::Internal)?;
    let path = match destination {
        EmailLink::VerifyEmail => account_auth::VERIFY_EMAIL_ROUTE,
        EmailLink::ResetPassword => account_auth::PASSWORD_RESET_PAGE_PATH,
        EmailLink::AdministratorInvitation => admin_auth::ADMIN_INVITATION_ACCEPT_PAGE_PATH,
        EmailLink::AdministratorReset => admin_auth::ADMIN_PASSWORD_RESET_PAGE_PATH,
        EmailLink::AgentClaim => agents::AGENT_OPERATOR_CLAIM_PAGE_PATH,
    };
    let mut url = Url::parse(origin).map_err(|_| AppError::Internal)?;
    url.set_path(path);
    url.query_pairs_mut()
        .append_pair(account_auth::ONE_TIME_TOKEN_QUERY_PARAMETER, token);
    Ok(url.into())
}

#[cfg(test)]
mod tests {
    use super::{EmailLink, build_email_link};

    #[test]
    fn links_use_only_trusted_origins_and_known_routes() {
        let token = "a".repeat(64);
        let link =
            build_email_link("https://example.test/", EmailLink::VerifyEmail, &token).unwrap();
        assert_eq!(
            link,
            format!("https://example.test/api/v1/auth/verify-email?token={token}")
        );
        assert!(
            build_email_link("https://user@evil.test", EmailLink::VerifyEmail, &token).is_err()
        );
        assert!(
            build_email_link("https://example.test/path", EmailLink::VerifyEmail, &token).is_err()
        );
        assert!(
            build_email_link(
                "https://example.test",
                EmailLink::VerifyEmail,
                "bad&next=evil"
            )
            .is_err()
        );
    }
}
