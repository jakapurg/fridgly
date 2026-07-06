//! Request handlers for the meals feature.
//!
//! `page` renders the full screen; `suggest` is the htmx endpoint that loads
//! the in-fridge items, asks the [`MealSuggester`](fridgly_domain::MealSuggester)
//! for ideas, and returns a fragment. Failures are rendered as friendly notices
//! (not HTTP errors) so the swap always shows something useful in-place.

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Form;
use fridgly_domain::{MealError, MealType};
use serde::Deserialize;

use super::view::{MealsPageTemplate, SuggestionVm, SuggestionsTemplate};
use crate::i18n::{ReqLocale, Ui};
use crate::state::AppState;

/// Full page: the meal-type picker plus an empty results area.
pub async fn page(ReqLocale(locale): ReqLocale) -> Response {
    MealsPageTemplate {
        active_tab: "meals",
        t: Ui::for_locale(locale),
    }
    .into_response()
}

#[derive(Deserialize)]
pub struct SuggestForm {
    #[serde(default)]
    meal_type: String,
}

/// Generate meal ideas for the chosen meal type and return the fragment.
pub async fn suggest(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Form(form): Form<SuggestForm>,
) -> Response {
    let t = Ui::for_locale(locale);
    let meal_type = MealType::from_key(&form.meal_type).unwrap_or(MealType::Lunch);

    let items = match state.items().list_in_fridge().await {
        Ok(items) => items,
        Err(_) => return notice(t, t.meals_error),
    };
    if items.is_empty() {
        return notice(t, t.meals_empty);
    }

    match state
        .meals()
        .suggest(&items, state.today(), meal_type, locale)
        .await
    {
        Ok(suggestions) => SuggestionsTemplate {
            t,
            suggestions: suggestions.into_iter().map(SuggestionVm::new).collect(),
            error: None,
        }
        .into_response(),
        Err(MealError::Unavailable) => notice(t, t.meals_unavailable),
        Err(MealError::NoItems) => notice(t, t.meals_empty),
        Err(MealError::Backend(msg)) => {
            tracing::error!(error = %msg, "meal suggestion failed");
            notice(t, t.meals_error)
        }
    }
}

/// Render the suggestions fragment carrying a localized notice instead of a list.
fn notice(t: Ui, message: &str) -> Response {
    SuggestionsTemplate {
        t,
        suggestions: Vec::new(),
        error: Some(message.to_string()),
    }
    .into_response()
}
