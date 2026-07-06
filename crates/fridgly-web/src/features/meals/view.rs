//! View models and templates for the meals feature.

use askama::Template;
use fridgly_domain::MealSuggestion;

use crate::i18n::Ui;

/// Presentation wrapper around a domain [`MealSuggestion`].
pub struct SuggestionVm {
    suggestion: MealSuggestion,
}

impl SuggestionVm {
    pub fn new(suggestion: MealSuggestion) -> Self {
        Self { suggestion }
    }

    pub fn title(&self) -> &str {
        &self.suggestion.title
    }
    pub fn description(&self) -> &str {
        &self.suggestion.description
    }
    /// Fridge items the idea uses, for the "uses what you have" tags.
    pub fn uses(&self) -> &[String] {
        &self.suggestion.uses
    }
}

/// The full meal-ideas page: the meal-type picker and the (initially empty)
/// results area that htmx swaps suggestions into.
#[derive(Template)]
#[template(path = "meals.html")]
pub struct MealsPageTemplate {
    pub active_tab: &'static str,
    pub t: Ui,
}

/// The suggestions fragment swapped in after a request. Exactly one of the
/// three states renders: an error/notice message, an empty result, or the list.
#[derive(Template)]
#[template(path = "meal_suggestions.html")]
pub struct SuggestionsTemplate {
    pub t: Ui,
    pub suggestions: Vec<SuggestionVm>,
    /// A localized notice shown instead of results (empty fridge, backend
    /// error, or "not configured").
    pub error: Option<String>,
}
