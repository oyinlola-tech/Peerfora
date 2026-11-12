use std::{collections::HashMap, env, net::IpAddr};

#[derive(Clone, serde::Deserialize)]
pub struct CountryPolicy {
    pub minimum_age: u8,
    pub policy_version: String,
    #[serde(default)]
    pub age_assurance_required: bool,
}

#[derive(Clone)]
pub struct OidcProviderSettings {
    pub client_id: String,
    pub client_secret: String,
    pub callback_url: String,
}

#[derive(Clone)]
pub struct Settings {
    pub host: IpAddr,
    pub port: u16,
    pub log_filter: String,
    pub database_url: String,
    pub cookie_secure: bool,
    pub country_policies: HashMap<String, CountryPolicy>,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: Option<String>,
    pub public_url: String,
    pub google_oidc: Option<OidcProviderSettings>,
    pub apple_oidc: Option<OidcProviderSettings>,
}

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let host = env::var("PEERFORA_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PEERFORA_PORT").unwrap_or_else(|_| "3000".to_owned());
    let log_filter = env::var("PEERFORA_LOG").unwrap_or_else(|_| "info".to_owned());
    let database_url = env::var("DATABASE_URL").unwrap_or_default();
    let production_environment = env::var("PEERFORA_ENV")
        .is_ok_and(|environment| environment.eq_ignore_ascii_case("production"));
    let cookie_secure = env::var("PEERFORA_COOKIE_SECURE")
        .map(|value| value == "true")
        .unwrap_or(false)
        || production_environment;
    let country_policies = env::var("PEERFORA_COUNTRY_POLICIES")
        .map(|value| serde_json::from_str(&value))
        .transpose()?
        .unwrap_or_default();
    let smtp_host = read_optional_setting("PEERFORA_SMTP_HOST");
    let smtp_port = env::var("PEERFORA_SMTP_PORT").unwrap_or_else(|_| "587".to_owned());
    let smtp_username = read_optional_setting("PEERFORA_SMTP_USERNAME");
    let smtp_password = read_optional_setting("PEERFORA_SMTP_PASSWORD");
    let smtp_from = read_optional_setting("PEERFORA_SMTP_FROM");
    let public_url =
        env::var("PEERFORA_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned());
    let google_oidc = load_oidc_provider("PEERFORA_GOOGLE", "google")?;
    let apple_oidc = load_oidc_provider("PEERFORA_APPLE", "apple")?;

    Ok(Settings {
        host: host.parse()?,
        port: port.parse()?,
        log_filter,
        database_url,
        cookie_secure,
        country_policies,
        smtp_host,
        smtp_port: smtp_port.parse()?,
        smtp_username,
        smtp_password,
        smtp_from,
        public_url,
        google_oidc,
        apple_oidc,
    })
}

fn load_oidc_provider(
    prefix: &str,
    provider_name: &str,
) -> Result<Option<OidcProviderSettings>, Box<dyn std::error::Error>> {
    let client_id = read_optional_setting(&format!("{prefix}_CLIENT_ID"));
    let client_secret = read_optional_setting(&format!("{prefix}_CLIENT_SECRET"));
    let callback_url = read_optional_setting(&format!("{prefix}_CALLBACK_URL"));
    match (client_id, client_secret, callback_url) {
        (Some(client_id), Some(client_secret), Some(callback_url)) => {
            Ok(Some(OidcProviderSettings {
                client_id,
                client_secret,
                callback_url,
            }))
        }
        (None, None, None) => Ok(None),
        _ => Err(format!(
            "{provider_name} sign-in needs client ID, client secret, and callback URL"
        )
        .into()),
    }
}

fn read_optional_setting(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}
