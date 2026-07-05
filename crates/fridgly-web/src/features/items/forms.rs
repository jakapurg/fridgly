//! Request payloads (DTOs) for the items feature and their translation into
//! validated domain value objects.

use chrono::NaiveDate;
use fridgly_domain::{DomainError, ItemChanges, NewItem};
use serde::Deserialize;

/// Raw form fields submitted by the browser. All optional strings because HTML
/// forms send empty strings rather than omitting fields.
#[derive(Debug, Deserialize)]
pub struct ItemForm {
    pub name: String,
    pub quantity: Option<String>,
    pub category: Option<String>,
    pub expiry_date: Option<String>,
}

impl ItemForm {
    fn expiry(&self) -> Option<NaiveDate> {
        self.expiry_date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// Convert into a validated [`NewItem`].
    pub fn into_new_item(self) -> Result<NewItem, DomainError> {
        let expiry = self.expiry();
        NewItem::new(self.name, self.quantity, self.category, expiry)
    }

    /// Convert into validated [`ItemChanges`].
    pub fn into_changes(self) -> Result<ItemChanges, DomainError> {
        let expiry = self.expiry();
        ItemChanges::new(self.name, self.quantity, self.category, expiry)
    }
}
