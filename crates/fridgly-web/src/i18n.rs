//! Web-layer internationalisation: the UI string catalog and per-request locale
//! detection.
//!
//! Locale is resolved in this order: the `lang` cookie (set via `/lang/:code`)
//! → the `Accept-Language` header → the default ([`Locale::default`], currently
//! Slovene). Expiry labels are localised in `fridgly-domain`; this module covers
//! the UI chrome.

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
    pub unit: &'static str,
    pub subunit: &'static str,
    pub subunit_unit: &'static str,
    pub left: &'static str,
    pub expires: &'static str,
    pub today: &'static str,
    pub plus_3d: &'static str,
    pub plus_1wk: &'static str,
    pub plus_1mo: &'static str,
    pub pick_date: &'static str,
    pub add_to_fridge: &'static str,
    pub search: &'static str,
    pub empty: &'static str,
    pub no_matches: &'static str,

    // Barcode scanning
    pub scan: &'static str,
    pub scan_title: &'static str,
    pub scan_hint: &'static str,
    pub scan_manual: &'static str,
    pub enter_barcode: &'static str,
    pub lookup: &'static str,
    pub scan_looking: &'static str,
    pub scan_notfound: &'static str,
    pub scan_error: &'static str,
    pub scan_unsupported: &'static str,

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
    pub meals_loading: &'static str,
    pub meals_uses: &'static str,
    pub meals_empty: &'static str,
    pub meals_no_ideas: &'static str,
    pub meals_error: &'static str,
    pub meals_unavailable: &'static str,

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
    unit: "e.g. packet",
    subunit: "Contains",
    subunit_unit: "e.g. eggs",
    left: "left",
    expires: "Expires",
    today: "Today",
    plus_3d: "+3d",
    plus_1wk: "+1wk",
    plus_1mo: "+1mo",
    pick_date: "📅 pick date",
    add_to_fridge: "Add to fridge",
    search: "Search",
    empty: "Fridge is empty. Add something above 👆",
    no_matches: "No matches",
    scan: "Scan barcode",
    scan_title: "Scan barcode",
    scan_hint: "Point your camera at a product barcode",
    scan_manual: "Or enter the barcode by hand",
    enter_barcode: "Barcode number",
    lookup: "Look up",
    scan_looking: "Looking up product…",
    scan_notfound: "Product not found. Try again or add it manually.",
    scan_error: "Lookup failed. Check your connection and try again.",
    scan_unsupported: "Camera scanning isn't available here. Enter the barcode below.",
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
    meals_loading: "Thinking up ideas…",
    meals_uses: "Uses",
    meals_empty: "Your fridge is empty. Add some items first 🧊",
    meals_no_ideas: "No ideas this time. Try a different meal.",
    meals_error: "Couldn't get suggestions right now. Please try again.",
    meals_unavailable: "Meal suggestions aren't set up yet.",
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
    unit: "npr. zavitek",
    subunit: "Vsebuje",
    subunit_unit: "npr. jajca",
    left: "ostalo",
    expires: "Poteče",
    today: "Danes",
    plus_3d: "+3d",
    plus_1wk: "+1t",
    plus_1mo: "+1m",
    pick_date: "📅 izberi datum",
    add_to_fridge: "Dodaj v hladilnik",
    search: "Išči",
    empty: "Hladilnik je prazen. Dodaj kaj zgoraj 👆",
    no_matches: "Ni zadetkov",
    scan: "Skeniraj črtno kodo",
    scan_title: "Skeniraj črtno kodo",
    scan_hint: "Usmeri kamero v črtno kodo izdelka",
    scan_manual: "Ali vnesi črtno kodo ročno",
    enter_barcode: "Številka črtne kode",
    lookup: "Poišči",
    scan_looking: "Iščem izdelek…",
    scan_notfound: "Izdelka ni bilo mogoče najti. Poskusi znova ali dodaj ročno.",
    scan_error: "Iskanje ni uspelo. Preveri povezavo in poskusi znova.",
    scan_unsupported: "Skeniranje s kamero tukaj ni na voljo. Vnesi črtno kodo spodaj.",
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
    meals_loading: "Razmišljam o idejah…",
    meals_uses: "Uporabi",
    meals_empty: "Hladilnik je prazen. Najprej dodaj kaj 🧊",
    meals_no_ideas: "Tokrat ni idej. Poskusi drug obrok.",
    meals_error: "Predlogov trenutno ni bilo mogoče pridobiti. Poskusi znova.",
    meals_unavailable: "Predlogi za jedi še niso nastavljeni.",
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
