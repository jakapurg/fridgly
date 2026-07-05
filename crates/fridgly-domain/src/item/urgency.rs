//! Expiry rules: how close an item is to going off, and a presentation-friendly
//! view of an item that carries those derived values.

use chrono::NaiveDate;

use super::model::Item;
use crate::locale::Locale;

/// Thresholds (in days from today) that separate the urgency bands.
const SOON_DAYS: i64 = 2;
const WEEK_DAYS: i64 = 7;

/// How close an item is to expiring — drives the colour band in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    /// Past its expiry date.
    Expired,
    /// Due within [`SOON_DAYS`] days.
    Soon,
    /// Due within [`WEEK_DAYS`] days.
    Week,
    /// Due later than a week out.
    Later,
    /// No expiry date recorded.
    NoDate,
}

impl Urgency {
    /// Classify an item's expiry relative to `today`.
    pub fn classify(expiry: Option<NaiveDate>, today: NaiveDate) -> Self {
        match expiry {
            None => Urgency::NoDate,
            Some(date) => {
                let days = (date - today).num_days();
                if days < 0 {
                    Urgency::Expired
                } else if days <= SOON_DAYS {
                    Urgency::Soon
                } else if days <= WEEK_DAYS {
                    Urgency::Week
                } else {
                    Urgency::Later
                }
            }
        }
    }

    /// Stable machine-readable key, used as a CSS class suffix in templates.
    pub fn key(self) -> &'static str {
        match self {
            Urgency::Expired => "expired",
            Urgency::Soon => "soon",
            Urgency::Week => "week",
            Urgency::Later => "later",
            Urgency::NoDate => "nodate",
        }
    }
}

/// An [`Item`] decorated with derived, display-oriented values.
///
/// This lives in the domain (not the web layer) because the urgency label is a
/// business rule, not a rendering detail.
#[derive(Debug, Clone)]
pub struct ItemView {
    pub item: Item,
    pub urgency: Urgency,
    /// Human label such as "today", "in 5d", "expired 2d ago".
    pub expiry_label: String,
}

impl ItemView {
    pub fn new(item: Item, today: NaiveDate, locale: Locale) -> Self {
        let urgency = Urgency::classify(item.expiry_date, today);
        let expiry_label = expiry_label(item.expiry_date, today, locale);
        Self {
            item,
            urgency,
            expiry_label,
        }
    }
}

/// Localised, human-readable expiry label relative to `today`.
fn expiry_label(expiry: Option<NaiveDate>, today: NaiveDate, locale: Locale) -> String {
    let Some(date) = expiry else {
        return match locale {
            Locale::En => "no date",
            Locale::Sl => "brez datuma",
        }
        .to_string();
    };
    let days = (date - today).num_days();
    match locale {
        Locale::En => match days {
            d if d < -1 => format!("expired {}d ago", -d),
            -1 => "expired yesterday".to_string(),
            0 => "today".to_string(),
            1 => "tomorrow".to_string(),
            d => format!("in {}d", d),
        },
        Locale::Sl => match days {
            d if d < -1 => format!("poteklo pred {}d", -d),
            -1 => "poteklo včeraj".to_string(),
            0 => "danes".to_string(),
            1 => "jutri".to_string(),
            d => format!("čez {}d", d),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn classifies_each_band() {
        let today = day("2026-07-05");
        assert_eq!(Urgency::classify(None, today), Urgency::NoDate);
        assert_eq!(
            Urgency::classify(Some(day("2026-07-04")), today),
            Urgency::Expired
        );
        assert_eq!(
            Urgency::classify(Some(day("2026-07-05")), today),
            Urgency::Soon
        );
        assert_eq!(
            Urgency::classify(Some(day("2026-07-07")), today),
            Urgency::Soon
        );
        assert_eq!(
            Urgency::classify(Some(day("2026-07-10")), today),
            Urgency::Week
        );
        assert_eq!(
            Urgency::classify(Some(day("2026-08-01")), today),
            Urgency::Later
        );
    }

    #[test]
    fn labels_read_naturally_in_english() {
        let today = day("2026-07-05");
        let en = Locale::En;
        assert_eq!(expiry_label(None, today, en), "no date");
        assert_eq!(expiry_label(Some(day("2026-07-05")), today, en), "today");
        assert_eq!(expiry_label(Some(day("2026-07-06")), today, en), "tomorrow");
        assert_eq!(expiry_label(Some(day("2026-07-10")), today, en), "in 5d");
        assert_eq!(
            expiry_label(Some(day("2026-07-04")), today, en),
            "expired yesterday"
        );
        assert_eq!(
            expiry_label(Some(day("2026-07-01")), today, en),
            "expired 4d ago"
        );
    }

    #[test]
    fn labels_translate_to_slovene() {
        let today = day("2026-07-05");
        let sl = Locale::Sl;
        assert_eq!(expiry_label(None, today, sl), "brez datuma");
        assert_eq!(expiry_label(Some(day("2026-07-05")), today, sl), "danes");
        assert_eq!(expiry_label(Some(day("2026-07-06")), today, sl), "jutri");
        assert_eq!(expiry_label(Some(day("2026-07-10")), today, sl), "čez 5d");
        assert_eq!(
            expiry_label(Some(day("2026-07-01")), today, sl),
            "poteklo pred 4d"
        );
    }
}
