//! Request handlers for the items feature.
//!
//! Handlers are deliberately thin: parse input → call the repository → render a
//! template fragment. Mutations re-render the whole list so htmx swaps in a
//! correctly re-sorted view. Every handler resolves the active locale so both
//! the expiry labels and UI chrome render in the user's language.

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Form;
use fridgly_domain::{ItemStatus, ItemView, Locale};
use uuid::Uuid;

use super::forms::ItemForm;
use super::view::{EditTemplate, IndexTemplate, ItemVm, ListTemplate, RowTemplate};
use crate::error::AppError;
use crate::i18n::{ReqLocale, Ui};
use crate::state::AppState;

type Handler = Result<Response, AppError>;

/// Full page: the fridge list plus the add sheet.
pub async fn index(State(state): State<AppState>, ReqLocale(locale): ReqLocale) -> Handler {
    let items = load_view_models(&state, locale).await?;
    Ok(IndexTemplate {
        active_tab: "fridge",
        t: Ui::for_locale(locale),
        items,
    }
    .into_response())
}

/// Add a new item, then return the refreshed list.
pub async fn create(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Form(form): Form<ItemForm>,
) -> Handler {
    let new_item = form.into_new_item()?;
    state.items().create(new_item).await?;
    render_list(&state, locale).await
}

/// Save edits to an item, then return the refreshed list.
pub async fn update(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Path(id): Path<Uuid>,
    Form(form): Form<ItemForm>,
) -> Handler {
    let changes = form.into_changes()?;
    state.items().update(id, changes).await?;
    render_list(&state, locale).await
}

/// Mark an item consumed.
pub async fn mark_used(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Path(id): Path<Uuid>,
) -> Handler {
    state.items().set_status(id, ItemStatus::Used).await?;
    render_list(&state, locale).await
}

/// Mark an item thrown away.
pub async fn mark_tossed(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Path(id): Path<Uuid>,
) -> Handler {
    state.items().set_status(id, ItemStatus::Tossed).await?;
    render_list(&state, locale).await
}

/// Permanently delete an item.
pub async fn delete(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Path(id): Path<Uuid>,
) -> Handler {
    state.items().delete(id).await?;
    render_list(&state, locale).await
}

/// Return the inline edit form for a single row.
pub async fn edit_form(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Path(id): Path<Uuid>,
) -> Handler {
    let vm = load_one(&state, id, locale).await?;
    Ok(EditTemplate {
        t: Ui::for_locale(locale),
        v: vm,
    }
    .into_response())
}

/// Return the plain (non-editing) row — used to cancel an edit.
pub async fn row(
    State(state): State<AppState>,
    ReqLocale(locale): ReqLocale,
    Path(id): Path<Uuid>,
) -> Handler {
    let vm = load_one(&state, id, locale).await?;
    Ok(RowTemplate {
        t: Ui::for_locale(locale),
        v: vm,
    }
    .into_response())
}

// ---- Helpers ----

async fn render_list(state: &AppState, locale: Locale) -> Handler {
    let items = load_view_models(state, locale).await?;
    Ok(ListTemplate {
        t: Ui::for_locale(locale),
        items,
    }
    .into_response())
}

async fn load_view_models(state: &AppState, locale: Locale) -> Result<Vec<ItemVm>, AppError> {
    let today = state.today();
    let items = state.items().list_in_fridge().await?;
    Ok(items
        .into_iter()
        .map(|item| ItemVm::new(ItemView::new(item, today, locale)))
        .collect())
}

async fn load_one(state: &AppState, id: Uuid, locale: Locale) -> Result<ItemVm, AppError> {
    let item = state.items().find(id).await?.ok_or(AppError::NotFound)?;
    Ok(ItemVm::new(ItemView::new(item, state.today(), locale)))
}
