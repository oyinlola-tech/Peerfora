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
            Self::Google => "https://accounts.google.com",
            Self::Apple => "https://appleid.apple.com",
        }
    }
}
