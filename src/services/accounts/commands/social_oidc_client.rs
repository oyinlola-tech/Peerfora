use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, RedirectUrl,
    core::{CoreClient, CoreProviderMetadata},
    reqwest,
};
use std::time::Duration;

use crate::{
    configs::settings::OidcProviderSettings,
    enums::{account_auth_error::AccountAuthError, social_provider::SocialProvider},
    errors::app_error::AppError,
};

pub async fn create_openid_client(
    provider: &SocialProvider,
    provider_settings: &OidcProviderSettings,
) -> Result<
    CoreClient<
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointMaybeSet,
        EndpointMaybeSet,
    >,
    AppError,
> {
    let http_client = create_openid_http_client()?;
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(provider.issuer_url().to_owned()).map_err(|_| AppError::Internal)?,
        &http_client,
    )
    .await
    .map_err(|_| AppError::from(AccountAuthError::SocialProviderUnavailable))?;
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(provider_settings.client_id.clone()),
        Some(ClientSecret::new(provider_settings.client_secret.clone())),
    );
    let callback_url =
        RedirectUrl::new(provider_settings.callback_url.clone()).map_err(|_| AppError::Internal)?;
    Ok(client.set_redirect_uri(callback_url))
}

pub fn create_openid_http_client() -> Result<reqwest::Client, AppError> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(
            crate::constants::account_auth::OIDC_HTTP_TIMEOUT_SECONDS,
        ))
        .timeout(Duration::from_secs(
            crate::constants::account_auth::OIDC_HTTP_TIMEOUT_SECONDS,
        ))
        .build()
        .map_err(|_| AppError::Internal)
}
