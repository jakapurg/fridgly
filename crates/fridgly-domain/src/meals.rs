//! Meal-idea suggestions from what's in the fridge.
//!
//! The domain owns the [`MealSuggester`] *port* and its value types; an adapter
//! in `fridgly-infra` fulfils it (today by asking an LLM). Keeping this as a
//! trait means the suggestion backend — an AI model, a recipe API, or a fake in
//! tests — can change without touching the web layer, exactly like
//! [`crate::ItemRepository`].
//!
//! Why AI rather than a recipe API: Fridgly defaults to Slovene and stores
//! free-form item names/quantities ("mleko", "2 vrečki", "ostanki piščanca").
//! Ingredient-search APIs (Spoonacular, Edamam, TheMealDB) are English-centric
//! and expect canonical ingredient IDs, so they match this data poorly. A
//! language model understands the fridge as written, prioritises soon-to-expire
//! items, and replies in the user's language.

use async_trait::async_trait;
use chrono::NaiveDate;
use thiserror::Error;

use crate::item::Item;
use crate::locale::Locale;

/// Which meal the suggestions should target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

impl MealType {
    /// Stable machine-readable key — used as the form value and CSS hook.
    pub fn key(self) -> &'static str {
        match self {
            MealType::Breakfast => "breakfast",
            MealType::Lunch => "lunch",
            MealType::Dinner => "dinner",
            MealType::Snack => "snack",
        }
    }

    /// Parse a key produced by [`MealType::key`] (case-insensitive).
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim().to_ascii_lowercase().as_str() {
            "breakfast" => Some(MealType::Breakfast),
            "lunch" => Some(MealType::Lunch),
            "dinner" => Some(MealType::Dinner),
            "snack" => Some(MealType::Snack),
            _ => None,
        }
    }
}

/// A single meal idea returned by a [`MealSuggester`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MealSuggestion {
    /// Short dish name.
    pub title: String,
    /// One or two sentences on what it is / how to make it.
    pub description: String,
    /// Fridge items the idea uses, echoed back for the "uses what you have" tags.
    pub uses: Vec<String>,
}

/// Failure modes for suggesting meals.
#[derive(Debug, Error)]
pub enum MealError {
    /// No suggestion backend is configured (e.g. missing API key).
    #[error("meal suggestions are not configured")]
    Unavailable,

    /// There is nothing in the fridge to build suggestions from.
    #[error("no items to suggest from")]
    NoItems,

    /// The backend failed (network, API error, unparseable response, …).
    #[error("meal suggestion failed: {0}")]
    Backend(String),
}

/// The suggestion *port*: the domain defines it, infrastructure implements it.
///
/// Implementations must be `Send + Sync` so they can be shared across async
/// request handlers behind an `Arc`.
#[async_trait]
pub trait MealSuggester: Send + Sync + 'static {
    /// Suggest a handful of meal ideas for `meal_type` from the current fridge
    /// `items`, prioritising those expiring soonest (relative to `today`) and
    /// answering in `locale`.
    async fn suggest(
        &self,
        items: &[Item],
        today: NaiveDate,
        meal_type: MealType,
        locale: Locale,
    ) -> Result<Vec<MealSuggestion>, MealError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meal_type_key_roundtrips() {
        for m in [
            MealType::Breakfast,
            MealType::Lunch,
            MealType::Dinner,
            MealType::Snack,
        ] {
            assert_eq!(MealType::from_key(m.key()), Some(m));
        }
    }

    #[test]
    fn meal_type_from_key_is_lenient_and_rejects_unknown() {
        assert_eq!(MealType::from_key("  DINNER "), Some(MealType::Dinner));
        assert_eq!(MealType::from_key("brunch"), None);
    }
}
