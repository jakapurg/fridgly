//! Supported UI languages.
//!
//! The locale lives in the domain because expiry labels ("in 5d" / "čez 5d")
//! are generated here; see [`crate::item::ItemView`]. UI-chrome translations and
//! locale *detection* (cookies, `Accept-Language`) are the web layer's concern.

/// A language the app can render in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    En,
    Sl,
}

impl Locale {
    /// ISO 639-1 code, e.g. `"en"`.
    pub fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Sl => "sl",
        }
    }

    /// Parse a language code (case-insensitive, ignores region like `sl-SI`).
    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().split(['-', '_']).next() {
            Some("en") => Some(Locale::En),
            Some("sl") => Some(Locale::Sl),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codes_and_regions() {
        assert_eq!(Locale::from_code("en"), Some(Locale::En));
        assert_eq!(Locale::from_code("SL"), Some(Locale::Sl));
        assert_eq!(Locale::from_code("sl-SI"), Some(Locale::Sl));
        assert_eq!(Locale::from_code("de"), None);
    }

    #[test]
    fn default_is_english() {
        assert_eq!(Locale::default(), Locale::En);
    }
}
