use url::Url;

pub(crate) fn validate_public_url(value: &str, production: bool) -> Result<(), String> {
    let url = validate_http_url(value, production, "PEERFORA_PUBLIC_URL")?;
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err(
            "PEERFORA_PUBLIC_URL must be an origin without a path, query, or fragment".to_owned(),
        );
    }
    Ok(())
}

pub(super) fn validate_oidc_callback(
    value: &str,
    production: bool,
    setting_name: &str,
) -> Result<(), String> {
    let url = validate_http_url(value, production, setting_name)?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err(format!(
            "{setting_name} must not contain a query or fragment"
        ));
    }
    Ok(())
}

fn validate_http_url(value: &str, production: bool, setting_name: &str) -> Result<Url, String> {
    let invalid = || format!("{setting_name} must be an absolute HTTP(S) URL without credentials");
    if value != value.trim()
        || value
            .chars()
            .any(|character| character.is_control() || character == '\\')
    {
        return Err(invalid());
    }
    let url = Url::parse(value).map_err(|_| invalid())?;
    let authority = value
        .split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or_default())
        .ok_or_else(invalid)?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || authority.contains('@')
    {
        return Err(invalid());
    }
    if production && url.scheme() != "https" {
        return Err(format!("{setting_name} must use HTTPS in production"));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::{validate_oidc_callback, validate_public_url};

    #[test]
    fn public_url_must_be_a_safe_origin() {
        assert!(validate_public_url("http://localhost:3000", false).is_ok());
        assert!(validate_public_url("https://peerfora.example", true).is_ok());
        for url in [
            "http://peerfora.example",
            "https://user:secret@peerfora.example",
            "https://@peerfora.example",
            "https://peerfora.example/path",
            "https://peerfora.example/?next=https://evil.example",
            "https://peerfora.example/#fragment",
            "javascript:alert(1)",
            "https://peerfora.example\n",
            "https://peerfora.example\t.evil.example",
            "https://peerfora.example\\evil.example",
        ] {
            assert!(validate_public_url(url, true).is_err(), "{url}");
        }
    }

    #[test]
    fn oidc_callback_must_be_safe_and_production_https() {
        assert!(
            validate_oidc_callback("http://localhost:3000/callback", false, "CALLBACK").is_ok()
        );
        assert!(
            validate_oidc_callback("https://peerfora.example/auth/callback", true, "CALLBACK")
                .is_ok()
        );
        for url in [
            "http://peerfora.example/callback",
            "https://user@peerfora.example/callback",
            "https://peerfora.example/callback?next=bad",
            "https://peerfora.example/callback#bad",
            "/callback",
        ] {
            assert!(
                validate_oidc_callback(url, true, "CALLBACK").is_err(),
                "{url}"
            );
        }
    }
}
