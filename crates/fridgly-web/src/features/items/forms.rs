//! Request payloads (DTOs) for the items feature and their translation into
//! validated domain value objects.

use chrono::NaiveDate;
use fridgly_domain::{DomainError, ItemChanges, NewItem, Subunit};
use serde::Deserialize;

/// Raw form fields submitted by the browser. All optional strings because HTML
/// forms send empty strings rather than omitting fields.
#[derive(Debug, Deserialize)]
pub struct ItemForm {
    pub name: String,
    pub quantity: Option<String>,
    /// Outer/container unit, e.g. "packet".
    pub unit: Option<String>,
    /// Pieces remaining inside the container, e.g. "3".
    pub subunit_remaining: Option<String>,
    /// What those pieces are, e.g. "eggs".
    pub subunit_unit: Option<String>,
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

    /// Build the subunit from the raw pieces count + unit. An unparseable or
    /// empty count means "no subunit tracked"; the domain clamps negatives.
    fn subunit(&self) -> Option<Subunit> {
        let remaining = self
            .subunit_remaining
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse::<i32>().ok());
        Subunit::new(remaining, self.subunit_unit.clone())
    }

    /// Convert into a validated [`NewItem`].
    pub fn into_new_item(self) -> Result<NewItem, DomainError> {
        let expiry = self.expiry();
        let subunit = self.subunit();
        NewItem::new(
            self.name,
            self.quantity,
            self.unit,
            subunit,
            self.category,
            expiry,
        )
    }

    /// Convert into validated [`ItemChanges`].
    pub fn into_changes(self) -> Result<ItemChanges, DomainError> {
        let expiry = self.expiry();
        let subunit = self.subunit();
        ItemChanges::new(
            self.name,
            self.quantity,
            self.unit,
            subunit,
            self.category,
            expiry,
        )
    }
}
