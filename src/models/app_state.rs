use std::collections::HashMap;
use std::sync::Arc;

use axum::http::HeaderValue;
use sqlx::PgPool;

use crate::configs::settings::{CountryPolicy, OidcProviderSettings, Settings};
use crate::interfaces::email_sender::EmailSender;

#[derive(Clone)]
pub struct AppState {
    pub database: PgPool,
    pub country_policies: HashMap<String, CountryPolicy>,
    pub cookie_secure: bool,
    pub production_environment: bool,
    pub admin_mfa_encryption_key: Option<String>,
    pub authentication_rate_limit_hash_key: String,
    pub allowed_origins: Vec<HeaderValue>,
    pub request_timeout_seconds: u64,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: Option<String>,
    pub public_url: String,
    pub google_oidc: Option<OidcProviderSettings>,
    pub apple_oidc: Option<OidcProviderSettings>,
    pub email_sender: Option<Arc<dyn EmailSender>>,
}

impl AppState {
    pub fn from_settings(
        database: PgPool,
        settings: &Settings,
        email_sender: Option<Arc<dyn EmailSender>>,
    ) -> Self {
        Self {
            database,
            country_policies: settings.country_policies.clone(),
            cookie_secure: settings.cookie_secure,
            production_environment: settings.production_environment,
            admin_mfa_encryption_key: settings.admin_mfa_encryption_key.clone(),
            authentication_rate_limit_hash_key: settings.authentication_rate_limit_hash_key.clone(),
            allowed_origins: settings.allowed_origins.clone(),
            request_timeout_seconds: settings.request_timeout_seconds,
            smtp_host: settings.smtp_host.clone(),
            smtp_port: settings.smtp_port,
            smtp_username: settings.smtp_username.clone(),
            smtp_password: settings.smtp_password.clone(),
            smtp_from: settings.smtp_from.clone(),
            public_url: settings.public_url.clone(),
            google_oidc: settings.google_oidc.clone(),
            apple_oidc: settings.apple_oidc.clone(),
            email_sender,
        }
    }
}
