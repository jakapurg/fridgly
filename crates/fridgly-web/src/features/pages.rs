//! Static placeholder screens for tabs that aren't functional yet
//! (Meal ideas, Shopping list), plus the language switcher endpoint. These
//! exist so the bottom navigation from the wireframe is complete.

use askama::Template;
use axum::extract::Path;
use axum::http::header::{LOCATION, REFERER, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use fridgly_domain::Locale;

use crate::i18n::{ReqLocale, Ui};
use crate::state::AppState;

#[derive(Template)]
#[template(path = "meals.html")]
struct MealsTemplate {
    active_tab: &'static str,
    t: Ui,
}

#[derive(Template)]
#[template(path = "shopping.html")]
struct ShoppingTemplate {
    active_tab: &'static str,
    t: Ui,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/meals", get(meals))
        .route("/shopping", get(shopping))
        .route("/lang/:code", get(set_lang))
}

async fn meals(ReqLocale(locale): ReqLocale) -> Response {
    MealsTemplate {
        active_tab: "meals",
        t: Ui::for_locale(locale),
    }
    .into_response()
}

async fn shopping(ReqLocale(locale): ReqLocale) -> Response {
    ShoppingTemplate {
        active_tab: "shopping",
        t: Ui::for_locale(locale),
    }
    .into_response()
}

/// Persist the chosen language in a cookie and return the user to the page they
/// came from (or the home screen).
async fn set_lang(Path(code): Path<String>, headers: HeaderMap) -> Response {
    let locale = Locale::from_code(&code).unwrap_or_default();
    let cookie = format!(
        "lang={}; Path=/; Max-Age=31536000; SameSite=Lax",
        locale.code()
    );
    let location = same_origin_referer(&headers).unwrap_or_else(|| "/".to_string());
    (
        StatusCode::SEE_OTHER,
        [(SET_COOKIE, cookie), (LOCATION, location)],
    )
        .into_response()
}

/// Only follow the `Referer` when it is a relative/local path, to avoid being
/// used as an open redirect.
fn same_origin_referer(headers: &HeaderMap) -> Option<String> {
    let referer = headers.get(REFERER)?.to_str().ok()?;
    // Accept a same-origin absolute URL by reducing it to its path, or a bare path.
    if let Some(rest) = referer
        .strip_prefix("http://")
        .or_else(|| referer.strip_prefix("https://"))
    {
        // rest = "host[:port]/path..." — keep from the first '/'.
        return rest.find('/').map(|i| rest[i..].to_string());
    }
    if referer.starts_with('/') {
        return Some(referer.to_string());
    }
    None
}
