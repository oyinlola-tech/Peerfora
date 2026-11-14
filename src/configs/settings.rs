use std::{collections::HashMap, env, net::IpAddr, str::FromStr};

use axum::http::{HeaderValue, Uri};

use crate::constants::authentication_rate_limits::{
    AUTHENTICATION_RATE_LIMIT_HASH_KEY_MINIMUM_BYTES, AUTHENTICATION_RATE_LIMIT_HASH_KEY_SETTING,
};
use crate::constants::runtime_settings::{
    DEFAULT_LOG_FILTER, DEFAULT_PUBLIC_URL, DEFAULT_SERVER_HOST, DEFAULT_SERVER_PORT,
    DEFAULT_SMTP_PORT,
};
use crate::constants::security::{
    ALLOWED_ORIGINS_SEPARATOR, ALLOWED_ORIGINS_SETTING, DEFAULT_REQUEST_TIMEOUT_SECONDS,
    REQUEST_TIMEOUT_SECONDS_SETTING,
};

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
    pub production_environment: bool,
    pub admin_mfa_encryption_key: Option<String>,
    pub authentication_rate_limit_hash_key: String,
    pub country_policies: HashMap<String, CountryPolicy>,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: Option<String>,
    pub public_url: String,
    pub allowed_origins: Vec<HeaderValue>,
    pub request_timeout_seconds: u64,
    pub google_oidc: Option<OidcProviderSettings>,
    pub apple_oidc: Option<OidcProviderSettings>,
}

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let host = env::var("PEERFORA_HOST").unwrap_or_else(|_| DEFAULT_SERVER_HOST.to_owned());
    let port = env::var("PEERFORA_PORT").unwrap_or_else(|_| DEFAULT_SERVER_PORT.to_owned());
    let log_filter = env::var("PEERFORA_LOG").unwrap_or_else(|_| DEFAULT_LOG_FILTER.to_owned());
    let database_url = env::var("DATABASE_URL").unwrap_or_default();
    let production_environment = env::var("PEERFORA_ENV")
        .is_ok_and(|environment| environment.eq_ignore_ascii_case("production"));
    let admin_mfa_encryption_key = read_optional_setting("PEERFORA_ADMIN_MFA_ENCRYPTION_KEY");
    let authentication_rate_limit_hash_key = read_required_secret(
        AUTHENTICATION_RATE_LIMIT_HASH_KEY_SETTING,
        AUTHENTICATION_RATE_LIMIT_HASH_KEY_MINIMUM_BYTES,
    )?;
    let cookie_secure = env::var("PEERFORA_COOKIE_SECURE")
        .map(|value| value == "true")
        .unwrap_or(false)
        || production_environment;
    let country_policies = match env::var("PEERFORA_COUNTRY_POLICIES") {
        Ok(value) => serde_json::from_str(&value)?,
        Err(_) => HashMap::new(),
    };
    let smtp_host = read_optional_setting("PEERFORA_SMTP_HOST");
    let smtp_port = env::var("PEERFORA_SMTP_PORT").unwrap_or_else(|_| DEFAULT_SMTP_PORT.to_owned());
    let smtp_username = read_optional_setting("PEERFORA_SMTP_USERNAME");
    let smtp_password = read_optional_setting("PEERFORA_SMTP_PASSWORD");
    let smtp_from = read_optional_setting("PEERFORA_SMTP_FROM");
    let public_url =
        env::var("PEERFORA_PUBLIC_URL").unwrap_or_else(|_| DEFAULT_PUBLIC_URL.to_owned());
    let allowed_origins = load_allowed_origins(&public_url)?;
    let request_timeout_seconds = load_request_timeout_seconds()?;
    let google_oidc = load_oidc_provider("PEERFORA_GOOGLE", "google")?;
    let apple_oidc = load_oidc_provider("PEERFORA_APPLE", "apple")?;

    Ok(Settings {
        host: host.parse()?,
        port: port.parse()?,
        log_filter,
        database_url,
        cookie_secure,
        production_environment,
        admin_mfa_encryption_key,
        authentication_rate_limit_hash_key,
        country_policies,
        smtp_host,
        smtp_port: smtp_port.parse()?,
        smtp_username,
        smtp_password,
        smtp_from,
        public_url,
        allowed_origins,
        request_timeout_seconds,
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

fn read_required_secret(
    name: &str,
    minimum_length: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let secret = read_optional_setting(name).ok_or_else(|| format!("{name} is required"))?;
    if secret.len() < minimum_length {
        return Err(format!("{name} must contain at least {minimum_length} bytes").into());
    }
    Ok(secret)
}

fn load_allowed_origins(public_url: &str) -> Result<Vec<HeaderValue>, Box<dyn std::error::Error>> {
    let configured_origins = read_optional_setting(ALLOWED_ORIGINS_SETTING);
    let origins = configured_origins
        .map(|value| {
            value
                .split(ALLOWED_ORIGINS_SEPARATOR)
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![public_url.trim_end_matches('/').to_owned()]);

    if origins.is_empty() {
        return Err(format!("{ALLOWED_ORIGINS_SETTING} must contain at least one origin").into());
    }

    origins
        .into_iter()
        .map(parse_allowed_origin)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn load_request_timeout_seconds() -> Result<u64, Box<dyn std::error::Error>> {
    let timeout = env::var(REQUEST_TIMEOUT_SECONDS_SETTING)
        .unwrap_or_else(|_| DEFAULT_REQUEST_TIMEOUT_SECONDS.to_string());
    parse_request_timeout_seconds(&timeout).map_err(Into::into)
}

fn parse_request_timeout_seconds(timeout: &str) -> Result<u64, String> {
    let timeout_seconds = timeout
        .parse::<u64>()
        .map_err(|_| format!("{REQUEST_TIMEOUT_SECONDS_SETTING} must be an integer"))?;
    if timeout_seconds == 0 {
        return Err(format!(
            "{REQUEST_TIMEOUT_SECONDS_SETTING} must be greater than zero"
        ));
    }
    Ok(timeout_seconds)
}

fn parse_allowed_origin(origin: String) -> Result<HeaderValue, String> {
    let origin = origin.trim_end_matches('/').to_owned();
    let uri = Uri::from_str(&origin)
        .map_err(|_| format!("Invalid origin in {ALLOWED_ORIGINS_SETTING}"))?;
    let Some(scheme) = uri.scheme_str() else {
        return Err(format!(
            "Origin in {ALLOWED_ORIGINS_SETTING} must include http or https"
        ));
    };
    let has_allowed_scheme =
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https");
    let has_authority = uri
        .authority()
        .is_some_and(|authority| !authority.as_str().contains('@'));
    let has_origin_only_path =
        (uri.path().is_empty() || uri.path() == "/") && uri.query().is_none();

    if !has_allowed_scheme || !has_authority || !has_origin_only_path {
        return Err(format!(
            "{ALLOWED_ORIGINS_SETTING} entries must be HTTP or HTTPS origins without a path"
        ));
    }

    HeaderValue::from_str(&origin)
        .map_err(|_| format!("Invalid origin in {ALLOWED_ORIGINS_SETTING}"))
}

#[cfg(test)]
mod tests {
    use super::{parse_allowed_origin, parse_request_timeout_seconds};

    #[test]
    fn accepts_exact_http_origin() {
        assert!(parse_allowed_origin("https://peerfora.example".to_owned()).is_ok());
    }

    #[test]
    fn rejects_origin_with_path() {
        assert!(parse_allowed_origin("https://peerfora.example/admin".to_owned()).is_err());
    }

    #[test]
    fn rejects_wildcard_and_non_http_origins() {
        assert!(parse_allowed_origin("*".to_owned()).is_err());
        assert!(parse_allowed_origin("ftp://peerfora.example".to_owned()).is_err());
    }

    #[test]
    fn accepts_positive_request_timeout_seconds() {
        assert_eq!(parse_request_timeout_seconds("30"), Ok(30));
    }

    #[test]
    fn rejects_zero_or_non_integer_request_timeout_seconds() {
        assert!(parse_request_timeout_seconds("0").is_err());
        assert!(parse_request_timeout_seconds("slow").is_err());
    }
}
