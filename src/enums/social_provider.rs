use crate::constants::social_auth::{APPLE_OIDC_ISSUER, GOOGLE_OIDC_ISSUER};

pub enum SocialProvider {
    Google,
    Apple,
}

impl SocialProvider {
    pub fn from_path_segment(value: &str) -> Option<Self> {
        match value {
            "google" => Some(Self::Google),
            "apple" => Some(Self::Apple),
            _ => None,
        }
    }

    pub fn as_database_value(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Apple => "apple",
        }
    }

    pub fn issuer_url(&self) -> &'static str {
        match self {
            Self::Google => GOOGLE_OIDC_ISSUER,
            Self::Apple => APPLE_OIDC_ISSUER,
        }
    }
}
