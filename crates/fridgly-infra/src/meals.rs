//! [`MealSuggester`] adapters.
//!
//! [`ClaudeMealSuggester`] fulfils the port by asking Anthropic's Claude for a
//! few meal ideas built from the fridge's contents. We call the Messages API
//! directly over HTTP (there is no official Anthropic Rust SDK) and constrain
//! the reply with a JSON-schema `output_config.format`, so the response is
//! always a parseable list of suggestions.
//!
//! [`UnavailableMealSuggester`] is a null adapter wired in when no API key is
//! configured; it reports [`MealError::Unavailable`] so the UI can show a
//! "not set up yet" message instead of the server refusing to start.

use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
use fridgly_domain::{Item, Locale, MealError, MealSuggester, MealSuggestion, MealType};
use serde::Deserialize;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

/// How many ideas to ask for.
const SUGGESTION_COUNT: usize = 3;

/// Meal suggester backed by the Anthropic Messages API.
pub struct ClaudeMealSuggester {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl ClaudeMealSuggester {
    /// Build a suggester that authenticates with `api_key` and uses `model`
    /// (e.g. `claude-opus-4-8`).
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            api_key,
            model,
        }
    }
}

/// A null suggester used when no backend is configured.
pub struct UnavailableMealSuggester;

#[async_trait]
impl MealSuggester for UnavailableMealSuggester {
    async fn suggest(
        &self,
        _items: &[Item],
        _today: NaiveDate,
        _meal_type: MealType,
        _locale: Locale,
    ) -> Result<Vec<MealSuggestion>, MealError> {
        Err(MealError::Unavailable)
    }
}

#[async_trait]
impl MealSuggester for ClaudeMealSuggester {
    async fn suggest(
        &self,
        items: &[Item],
        today: NaiveDate,
        meal_type: MealType,
        locale: Locale,
    ) -> Result<Vec<MealSuggestion>, MealError> {
        if items.is_empty() {
            return Err(MealError::NoItems);
        }

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 2048,
            "system": system_prompt(locale),
            "messages": [{ "role": "user", "content": user_prompt(items, today, meal_type) }],
            "output_config": { "format": response_format() },
        });

        let response = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|e| MealError::Backend(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            return Err(MealError::Backend(format!(
                "anthropic API returned {status}: {detail}"
            )));
        }

        let message: ApiResponse = response
            .json()
            .await
            .map_err(|e| MealError::Backend(e.to_string()))?;

        if message.stop_reason.as_deref() == Some("refusal") {
            return Err(MealError::Backend("request was refused".to_string()));
        }

        // With a JSON-schema output format the model's reply is a single text
        // block whose text is the JSON document.
        let text = message
            .content
            .into_iter()
            .find(|block| block.block_type == "text")
            .map(|block| block.text)
            .ok_or_else(|| MealError::Backend("no text block in response".to_string()))?;

        let payload: SuggestionsPayload = serde_json::from_str(&text)
            .map_err(|e| MealError::Backend(format!("could not parse suggestions: {e}")))?;

        Ok(payload
            .suggestions
            .into_iter()
            .map(|s| MealSuggestion {
                title: s.title,
                description: s.description,
                uses: s.uses,
            })
            .collect())
    }
}

/// The full name of a language, for instructing the model which to reply in.
fn language_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "English",
        Locale::Sl => "Slovenian",
    }
}

fn system_prompt(locale: Locale) -> String {
    format!(
        "You are a helpful cooking assistant inside a fridge-tracking app. Given the food \
         currently in a household's fridge and a target meal, suggest {SUGGESTION_COUNT} simple \
         meal ideas. Strongly prefer ideas that use the items expiring soonest, to reduce food \
         waste, and that need few or no extra ingredients. Keep each description to one or two \
         short sentences. In the \"uses\" list for each idea, include only fridge items the idea \
         actually uses, copied from the list provided. Reply entirely in {language}.",
        language = language_name(locale)
    )
}

fn user_prompt(items: &[Item], today: NaiveDate, meal_type: MealType) -> String {
    let mut prompt = format!(
        "Meal: {meal}\nToday's date: {today}\nItems in the fridge (name — quantity — expiry):\n",
        meal = meal_type.key(),
    );
    for item in items {
        prompt.push_str(&format!(
            "- {} — {} — {}\n",
            item.name,
            item.quantity,
            expiry_hint(item.expiry_date, today)
        ));
    }
    prompt
}

/// A compact, language-neutral expiry description for the prompt.
fn expiry_hint(expiry: Option<NaiveDate>, today: NaiveDate) -> String {
    match expiry {
        None => "no expiry date".to_string(),
        Some(date) => {
            let days = (date - today).num_days();
            match days {
                d if d < 0 => format!("expired {} day(s) ago", -d),
                0 => "expires today".to_string(),
                1 => "expires tomorrow".to_string(),
                d => format!("expires in {d} day(s)"),
            }
        }
    }
}

/// JSON-schema constraint on the reply, guaranteeing a parseable shape.
fn response_format() -> serde_json::Value {
    serde_json::json!({
        "type": "json_schema",
        "schema": {
            "type": "object",
            "properties": {
                "suggestions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": { "type": "string" },
                            "description": { "type": "string" },
                            "uses": { "type": "array", "items": { "type": "string" } }
                        },
                        "required": ["title", "description", "uses"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["suggestions"],
            "additionalProperties": false
        }
    })
}

/// Top-level Messages API response (only the fields we read).
#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: String,
}

/// The JSON document the model returns inside the text block.
#[derive(Deserialize)]
struct SuggestionsPayload {
    suggestions: Vec<SuggestionDto>,
}

#[derive(Deserialize)]
struct SuggestionDto {
    title: String,
    description: String,
    #[serde(default)]
    uses: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[tokio::test]
    async fn unavailable_suggester_reports_unavailable() {
        let today = day("2026-07-06");
        let items = Vec::new();
        let result = UnavailableMealSuggester
            .suggest(&items, today, MealType::Lunch, Locale::Sl)
            .await;
        assert!(matches!(result, Err(MealError::Unavailable)));
    }

    #[test]
    fn expiry_hint_describes_each_case() {
        let today = day("2026-07-06");
        assert_eq!(expiry_hint(None, today), "no expiry date");
        assert_eq!(expiry_hint(Some(day("2026-07-06")), today), "expires today");
        assert_eq!(
            expiry_hint(Some(day("2026-07-07")), today),
            "expires tomorrow"
        );
        assert_eq!(
            expiry_hint(Some(day("2026-07-11")), today),
            "expires in 5 day(s)"
        );
        assert_eq!(
            expiry_hint(Some(day("2026-07-04")), today),
            "expired 2 day(s) ago"
        );
    }

    #[test]
    fn user_prompt_lists_items_with_expiry() {
        let today = day("2026-07-06");
        let items = vec![Item {
            id: uuid::Uuid::nil(),
            name: "Milk".to_string(),
            quantity: "1L".to_string(),
            unit: None,
            subunit: None,
            category: None,
            expiry_date: Some(day("2026-07-08")),
            status: fridgly_domain::ItemStatus::InFridge,
            added_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }];
        let prompt = user_prompt(&items, today, MealType::Dinner);
        assert!(prompt.contains("Meal: dinner"));
        assert!(prompt.contains("- Milk — 1L — expires in 2 day(s)"));
    }
}
