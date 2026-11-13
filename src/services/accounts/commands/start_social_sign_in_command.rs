use openidconnect::{CsrfToken, Nonce, PkceCodeChallenge, Scope, core::CoreAuthenticationFlow};
use sqlx::PgPool;

use crate::{
    configs::settings::OidcProviderSettings, enums::social_provider::SocialProvider,
    errors::app_error::AppError, repositories::social_auth_repository,
    services::accounts::commands::social_oidc_client::create_openid_client,
    utils::one_time_token::hash_one_time_token,
};

use super::social_auth_types::SocialAuthorizationStart;

pub async fn start_social_sign_in(
    pool: &PgPool,
    provider: &SocialProvider,
    provider_settings: &OidcProviderSettings,
) -> Result<SocialAuthorizationStart, AppError> {
    let client = create_openid_client(provider, provider_settings).await?;
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorization_url, csrf_state, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_owned()))
        .add_scope(Scope::new("email".to_owned()))
        .set_pkce_challenge(pkce_challenge)
        .url();
    let state = csrf_state.secret().clone();
    social_auth_repository::save_oauth_attempt(
        pool,
        &hash_one_time_token(&state),
        provider.as_database_value(),
        nonce.secret(),
        pkce_verifier.secret(),
    )
    .await?;
    Ok(SocialAuthorizationStart {
        authorization_url: authorization_url.to_string(),
        state,
    })
}
