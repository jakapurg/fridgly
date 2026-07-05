//! Web-layer internationalisation: the UI string catalog and per-request locale
//! detection.
//!
//! Locale is resolved in this order: the `lang` cookie (set by the language
//! switcher) → the `Accept-Language` header → English. Expiry labels are
//! localised in `fridgly-domain`; this module covers the UI chrome.

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::{ACCEPT_LANGUAGE, COOKIE};
use axum::http::request::Parts;
use axum::http::HeaderMap;
use fridgly_domain::Locale;
use std::convert::Infallible;

/// All translatable UI strings for one language. `Copy` because every field is
/// a `&'static str`, so it is cheap to hand to templates by value.
#[derive(Clone, Copy)]
pub struct Ui {
    pub lang: &'static str,

    // Navigation
    pub fridge: &'static str,
    pub meal_ideas: &'static str,
    pub shopping: &'static str,

    // Home / add sheet
    pub add_item: &'static str,
    pub name: &'static str,
    pub quantity: &'static str,
    pub expires: &'static str,
    pub today: &'static str,
    pub plus_3d: &'static str,
    pub plus_1wk: &'static str,
    pub plus_1mo: &'static str,
    pub pick_date: &'static str,
    pub add_to_fridge: &'static str,
    pub empty: &'static str,

    // Legend
    pub leg_expired: &'static str,
    pub leg_2d: &'static str,
    pub leg_week: &'static str,
    pub leg_later: &'static str,

    // Row + edit
    pub edit: &'static str,
    pub used_title: &'static str,
    pub toss_title: &'static str,
    pub toss_confirm: &'static str,
    pub cancel: &'static str,
    pub save: &'static str,

    // Meals
    pub what_meal: &'static str,
    pub breakfast: &'static str,
    pub lunch: &'static str,
    pub dinner: &'static str,
    pub snack: &'static str,
    pub suggest_meals: &'static str,
    pub meals_blurb: &'static str,

    // Shopping
    pub shopping_list: &'static str,
    pub running_low: &'static str,
    pub shopping_blurb: &'static str,

    // Shared
    pub coming_soon: &'static str,
}

impl Ui {
    pub fn for_locale(locale: Locale) -> Self {
        match locale {
            Locale::En => EN,
            Locale::Sl => SL,
        }
    }
}

const EN: Ui = Ui {
    lang: "en",
    fridge: "Fridge",
    meal_ideas: "Meal ideas",
    shopping: "Shopping",
    add_item: "Add item",
    name: "Name",
    quantity: "Quantity",
    expires: "Expires",
    today: "Today",
    plus_3d: "+3d",
    plus_1wk: "+1wk",
    plus_1mo: "+1mo",
    pick_date: "📅 pick date",
    add_to_fridge: "Add to fridge",
    empty: "Fridge is empty. Add something above 👆",
    leg_expired: "expired",
    leg_2d: "≤2d",
    leg_week: "this wk",
    leg_later: "later",
    edit: "Edit",
    used_title: "Mark used",
    toss_title: "Toss",
    toss_confirm: "Toss",
    cancel: "Cancel",
    save: "Save",
    what_meal: "What meal?",
    breakfast: "Breakfast",
    lunch: "Lunch",
    dinner: "Dinner",
    snack: "Snack",
    suggest_meals: "✨ Suggest meals",
    meals_blurb: "Suggestions will use what's in your fridge, prioritizing items expiring soon.",
    shopping_list: "Shopping list",
    running_low: "Running low / used up",
    shopping_blurb: "Items you mark used or tossed will collect here so you know what to restock.",
    coming_soon: "Coming soon",
};

const SL: Ui = Ui {
    lang: "sl",
    fridge: "Hladilnik",
    meal_ideas: "Ideje za jedi",
    shopping: "Nakupovanje",
    add_item: "Dodaj izdelek",
    name: "Ime",
    quantity: "Količina",
    expires: "Poteče",
    today: "Danes",
    plus_3d: "+3d",
    plus_1wk: "+1t",
    plus_1mo: "+1m",
    pick_date: "📅 izberi datum",
    add_to_fridge: "Dodaj v hladilnik",
    empty: "Hladilnik je prazen. Dodaj kaj zgoraj 👆",
    leg_expired: "poteklo",
    leg_2d: "≤2d",
    leg_week: "ta teden",
    leg_later: "kasneje",
    edit: "Uredi",
    used_title: "Označi porabljeno",
    toss_title: "Zavrzi",
    toss_confirm: "Zavržem",
    cancel: "Prekliči",
    save: "Shrani",
    what_meal: "Kateri obrok?",
    breakfast: "Zajtrk",
    lunch: "Kosilo",
    dinner: "Večerja",
    snack: "Prigrizek",
    suggest_meals: "✨ Predlagaj jedi",
    meals_blurb:
        "Predlogi bodo upoštevali, kar imaš v hladilniku, s prednostjo izdelkom, ki kmalu potečejo.",
    shopping_list: "Nakupovalni seznam",
    running_low: "Pošlo / porabljeno",
    shopping_blurb:
        "Izdelki, ki jih označiš kot porabljene ali zavržene, se bodo zbrali tukaj, da veš, kaj dokupiti.",
    coming_soon: "Kmalu na voljo",
};

/// Request extractor that resolves the active [`Locale`].
pub struct ReqLocale(pub Locale);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ReqLocale {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(ReqLocale(detect_locale(&parts.headers)))
    }
}

/// Cookie (`lang`) → `Accept-Language` → default.
fn detect_locale(headers: &HeaderMap) -> Locale {
    if let Some(locale) = cookie_locale(headers) {
        return locale;
    }
    accept_language_locale(headers).unwrap_or_default()
}

fn cookie_locale(headers: &HeaderMap) -> Option<Locale> {
    let cookies = headers.get(COOKIE)?.to_str().ok()?;
    cookies
        .split(';')
        .filter_map(|c| c.trim().split_once('='))
        .find(|(k, _)| *k == "lang")
        .and_then(|(_, v)| Locale::from_code(v))
}

fn accept_language_locale(headers: &HeaderMap) -> Option<Locale> {
    let header = headers.get(ACCEPT_LANGUAGE)?.to_str().ok()?;
    // Take languages in order, ignoring q-weights, and pick the first supported.
    header
        .split(',')
        .filter_map(|part| part.split(';').next())
        .find_map(Locale::from_code)
}
