use std::sync::OnceLock;

use serde::Deserialize;

use crate::errors::app_error::AppError;

const PROVISIONAL_THEME: &str = include_str!("../../../../packages/design-tokens/email-theme.json");
static THEME: OnceLock<Result<EmailTheme, ()>> = OnceLock::new();

#[derive(Deserialize)]
pub(super) struct EmailTheme {
    mode: String,
    pub brand_name: String,
    pub body_background: String,
    pub content_surface: String,
    pub primary_text: String,
    pub secondary_text: String,
    pub border: String,
    pub action_background: String,
    pub action_text: String,
    pub link: String,
    pub body_font: String,
    pub content_width_px: u16,
}

impl EmailTheme {
    fn parse(value: &str) -> Result<Self, ()> {
        let theme: Self = serde_json::from_str(value).map_err(|_| ())?;
        let colors = [
            &theme.body_background,
            &theme.content_surface,
            &theme.primary_text,
            &theme.secondary_text,
            &theme.border,
            &theme.action_background,
            &theme.action_text,
            &theme.link,
        ];
        if theme.mode != "provisional"
            || theme.brand_name.trim().is_empty()
            || theme.brand_name.len() > 80
            || theme.brand_name.chars().any(char::is_control)
            || colors.iter().any(|color| {
                color.len() != 7
                    || !color.starts_with('#')
                    || !color[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
            })
            || theme.body_font.is_empty()
            || !theme
                .body_font
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b',' | b'-'))
            || !(320..=800).contains(&theme.content_width_px)
        {
            return Err(());
        }
        Ok(theme)
    }
}

pub(super) fn provisional_theme() -> Result<&'static EmailTheme, AppError> {
    THEME
        .get_or_init(|| EmailTheme::parse(PROVISIONAL_THEME))
        .as_ref()
        .map_err(|_| AppError::Internal)
}

#[cfg(test)]
mod tests {
    use super::{EmailTheme, PROVISIONAL_THEME};

    #[test]
    fn theme_is_provisional_and_rejects_invalid_css() {
        assert!(EmailTheme::parse(PROVISIONAL_THEME).is_ok());
        assert!(
            EmailTheme::parse(&PROVISIONAL_THEME.replace("#f4f4f4", "red;display:none")).is_err()
        );
        assert!(
            EmailTheme::parse(&PROVISIONAL_THEME.replace("Arial, sans-serif", "Arial;url(bad)"))
                .is_err()
        );
        assert!(EmailTheme::parse(&PROVISIONAL_THEME.replace("provisional", "branded")).is_err());
    }
}
