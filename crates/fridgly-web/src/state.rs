//! Shared application state injected into every request handler.

use std::sync::Arc;

use chrono::{Local, NaiveDate};
use fridgly_domain::{ItemRepository, MealSuggester, ProductCatalog};

/// State handed to Axum handlers. Cheap to clone (everything behind an `Arc`).
///
/// Handlers depend on the [`ItemRepository`], [`ProductCatalog`] and
/// [`MealSuggester`] *trait objects* rather than concrete types, so the
/// storage backend, product data source and meal-idea backend can each be
/// swapped (or faked in tests) without touching the web layer.
#[derive(Clone)]
pub struct AppState {
    items: Arc<dyn ItemRepository>,
    catalog: Arc<dyn ProductCatalog>,
    meals: Arc<dyn MealSuggester>,
}

impl AppState {
    pub fn new(
        items: Arc<dyn ItemRepository>,
        catalog: Arc<dyn ProductCatalog>,
        meals: Arc<dyn MealSuggester>,
    ) -> Self {
        Self {
            items,
            catalog,
            meals,
        }
    }

    /// The item repository.
    pub fn items(&self) -> &dyn ItemRepository {
        self.items.as_ref()
    }

    /// The product catalog used for barcode lookups.
    pub fn catalog(&self) -> &dyn ProductCatalog {
        self.catalog.as_ref()
    }

    /// The meal-idea suggester.
    pub fn meals(&self) -> &dyn MealSuggester {
        self.meals.as_ref()
    }

    /// Today's date in the server's local timezone — the reference point for
    /// all expiry calculations.
    pub fn today(&self) -> NaiveDate {
        Local::now().date_naive()
    }
}
