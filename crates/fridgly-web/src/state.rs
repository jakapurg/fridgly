//! Shared application state injected into every request handler.

use std::sync::Arc;

use chrono::{Local, NaiveDate};
use fridgly_domain::ItemRepository;

/// State handed to Axum handlers. Cheap to clone (everything behind an `Arc`).
///
/// Handlers depend on the [`ItemRepository`] *trait object* rather than a
/// concrete type, so the storage backend can be swapped (or faked in tests)
/// without touching the web layer.
#[derive(Clone)]
pub struct AppState {
    items: Arc<dyn ItemRepository>,
}

impl AppState {
    pub fn new(items: Arc<dyn ItemRepository>) -> Self {
        Self { items }
    }

    /// The item repository.
    pub fn items(&self) -> &dyn ItemRepository {
        self.items.as_ref()
    }

    /// Today's date in the server's local timezone — the reference point for
    /// all expiry calculations.
    pub fn today(&self) -> NaiveDate {
        Local::now().date_naive()
    }
}
