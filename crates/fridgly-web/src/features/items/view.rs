//! View models and templates for the items feature.
//!
//! [`ItemVm`] adapts the domain [`ItemView`] to the accessors the Askama
//! templates call, keeping template logic thin and the domain type free of
//! presentation concerns.

use askama::Template;
use fridgly_domain::ItemView;
use uuid::Uuid;

use crate::i18n::Ui;

/// Presentation wrapper around a domain [`ItemView`].
pub struct ItemVm {
    view: ItemView,
}

impl ItemVm {
    pub fn new(view: ItemView) -> Self {
        Self { view }
    }

    pub fn id(&self) -> Uuid {
        self.view.item.id
    }
    pub fn name(&self) -> &str {
        &self.view.item.name
    }
    pub fn quantity(&self) -> &str {
        &self.view.item.quantity
    }
    /// Container unit (e.g. `"packet"`), or empty when none — used as an edit
    /// input value.
    pub fn unit(&self) -> &str {
        self.view.item.unit.as_deref().unwrap_or("")
    }
    /// Chip text combining quantity and unit, e.g. `"1 packet"` (or just the
    /// quantity when no unit is set).
    pub fn qty_display(&self) -> String {
        match &self.view.item.unit {
            Some(unit) => format!("{} {}", self.view.item.quantity, unit),
            None => self.view.item.quantity.clone(),
        }
    }
    /// Whether this item tracks remaining pieces inside its container.
    pub fn has_subunit(&self) -> bool {
        self.view.item.subunit.is_some()
    }
    /// Chip text for the remaining pieces, e.g. `"3 eggs"` or `"3"`.
    pub fn subunit_display(&self) -> String {
        match &self.view.item.subunit {
            Some(s) => match &s.unit {
                Some(unit) => format!("{} {}", s.remaining, unit),
                None => s.remaining.to_string(),
            },
            None => String::new(),
        }
    }
    /// Remaining-pieces count for an edit input (empty when not tracked).
    pub fn subunit_remaining_input(&self) -> String {
        self.view
            .item
            .subunit
            .as_ref()
            .map(|s| s.remaining.to_string())
            .unwrap_or_default()
    }
    /// Subunit unit label (e.g. `"eggs"`) for an edit input, or empty.
    pub fn subunit_unit(&self) -> &str {
        self.view
            .item
            .subunit
            .as_ref()
            .and_then(|s| s.unit.as_deref())
            .unwrap_or("")
    }
    pub fn category(&self) -> &str {
        self.view.item.category.as_deref().unwrap_or("")
    }
    /// Urgency band key, e.g. `"soon"`, used as a CSS class suffix.
    pub fn band(&self) -> &'static str {
        self.view.urgency.key()
    }
    pub fn expiry_label(&self) -> &str {
        &self.view.expiry_label
    }
    /// Value for an `<input type="date">` (`YYYY-MM-DD`, or empty).
    pub fn expiry_input(&self) -> String {
        self.view
            .item
            .expiry_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub active_tab: &'static str,
    pub t: Ui,
    pub is_search: bool,
    pub items: Vec<ItemVm>,
}

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
    pub t: Ui,
    /// True when the list is the result of a search, so an empty result shows
    /// "no matches" rather than "fridge is empty".
    pub is_search: bool,
    pub items: Vec<ItemVm>,
}

#[derive(Template)]
#[template(path = "item_row.html")]
pub struct RowTemplate {
    pub t: Ui,
    pub v: ItemVm,
}

#[derive(Template)]
#[template(path = "edit_form.html")]
pub struct EditTemplate {
    pub t: Ui,
    pub v: ItemVm,
}
