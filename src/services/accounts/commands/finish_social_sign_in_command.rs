use chrono::{Duration, Utc};
use openidconnect::{AuthorizationCode, Nonce, PkceCodeVerifier, TokenResponse};
use sqlx::PgPool;

use crate::{
    configs::settings::OidcProviderSettings,
    constants::account_auth::SESSION_LIFETIME_DAYS,
    enums::{account_auth_error::AccountAuthError, social_provider::SocialProvider},
    errors::app_error::AppError,
    models::account::Account,
    repositories::{account_repository, social_auth_repository},
    services::accounts::commands::social_oidc_client::{
        create_openid_client, create_openid_http_client,
    },
    utils::email_address::normalize_email_address,
    utils::one_time_token::{create_one_time_token, hash_one_time_token},
};

use super::social_auth_types::SocialAuthorizationResult;

pub async fn finish_social_sign_in(
    pool: &PgPool,
    provider: &SocialProvider,
    provider_settings: &OidcProviderSettings,
    state: &str,
    authorization_code: &str,
) -> Result<SocialAuthorizationResult, AppError> {
    let attempt = social_auth_repository::consume_oauth_attempt(
        pool,
        &hash_one_time_token(state),
        provider.as_database_value(),
    )
    .await?;
    let client = create_openid_client(provider, provider_settings).await?;
    let http_client = create_openid_http_client()?;
    let token_response = client
        .exchange_code(AuthorizationCode::new(authorization_code.to_owned()))
        .map_err(|_| AccountAuthError::SocialStateInvalid)?
        .set_pkce_verifier(PkceCodeVerifier::new(attempt.pkce_verifier))
        .request_async(&http_client)
        .await
        .map_err(|_| AccountAuthError::SocialStateInvalid)?;
    let id_token = token_response
        .id_token()
        .ok_or(AccountAuthError::SocialStateInvalid)?;
    let claims = id_token
        .claims(&client.id_token_verifier(), &Nonce::new(attempt.nonce))
        .map_err(|_| AccountAuthError::SocialStateInvalid)?;
    let email = claims
        .email()
        .filter(|_| claims.email_verified() == Some(true))
        .map(|email| normalize_email_address(email.as_str()))
        .ok_or(AccountAuthError::SocialEmailNotVerified)?;
    let provider_subject = claims.subject().as_str();

    if let Some(account) = social_auth_repository::find_account_by_social_identity(
        pool,
        provider.as_database_value(),
        provider_subject,
    )
    .await?
    {
        return create_social_session(pool, account).await;
    }

    if account_repository::find_account_by_email(pool, &email)
        .await?
        .is_some()
    {
        return Err(AccountAuthError::ExistingAccountNeedsLinking.into());
    }

    let onboarding_token = create_one_time_token();
    social_auth_repository::save_pending_social_account(
        pool,
        &hash_one_time_token(&onboarding_token),
        &social_auth_repository::PendingSocialAccount {
            provider: provider.as_database_value().to_owned(),
            provider_subject: provider_subject.to_owned(),
            email,
            display_name: None,
        },
    )
    .await?;
    Ok(SocialAuthorizationResult::NeedsBirthday { onboarding_token })
}

async fn create_social_session(
    pool: &PgPool,
    account: Account,
) -> Result<SocialAuthorizationResult, AppError> {
    let session_token = create_one_time_token();
    let expires_at = Utc::now() + Duration::days(SESSION_LIFETIME_DAYS);
    social_auth_repository::create_social_session(
        pool,
        account.id,
        &hash_one_time_token(&session_token),
        expires_at,
    )
    .await?;
    Ok(SocialAuthorizationResult::SignedIn {
        account,
        session_token,
    })
}
