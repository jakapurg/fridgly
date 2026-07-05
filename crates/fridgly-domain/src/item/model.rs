//! Core item entity and the value objects used to create/change it.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::DomainError;

/// Lifecycle state of an item. Items are never hard-deleted on normal use so we
/// can later report on how much food was eaten vs. thrown away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    /// Currently in the fridge.
    InFridge,
    /// Consumed.
    Used,
    /// Thrown away (expired / spoiled).
    Tossed,
}

impl ItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ItemStatus::InFridge => "in_fridge",
            ItemStatus::Used => "used",
            ItemStatus::Tossed => "tossed",
        }
    }
}

impl fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ItemStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in_fridge" => Ok(ItemStatus::InFridge),
            "used" => Ok(ItemStatus::Used),
            "tossed" => Ok(ItemStatus::Tossed),
            other => Err(format!("unknown item status: {other}")),
        }
    }
}

/// A persisted fridge item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub quantity: String,
    pub category: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub status: ItemStatus,
    pub added_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Validated data for creating a new item.
///
/// Construct via [`NewItem::new`] so invariants (e.g. non-empty name) are
/// enforced in one place.
#[derive(Debug, Clone)]
pub struct NewItem {
    pub name: String,
    pub quantity: String,
    pub category: Option<String>,
    pub expiry_date: Option<NaiveDate>,
}

impl NewItem {
    /// Build a validated `NewItem`, normalising whitespace and applying the
    /// default quantity of `"1"` when none is supplied.
    pub fn new(
        name: impl Into<String>,
        quantity: Option<String>,
        category: Option<String>,
        expiry_date: Option<NaiveDate>,
    ) -> Result<Self, DomainError> {
        let name = name.into().trim().to_string();
        if name.is_empty() {
            return Err(DomainError::Required { field: "name" });
        }
        Ok(Self {
            name,
            quantity: normalize_quantity(quantity),
            category: normalize_optional(category),
            expiry_date,
        })
    }
}

/// Validated data for editing an existing item (full replace of editable fields).
#[derive(Debug, Clone)]
pub struct ItemChanges {
    pub name: String,
    pub quantity: String,
    pub category: Option<String>,
    pub expiry_date: Option<NaiveDate>,
}

impl ItemChanges {
    pub fn new(
        name: impl Into<String>,
        quantity: Option<String>,
        category: Option<String>,
        expiry_date: Option<NaiveDate>,
    ) -> Result<Self, DomainError> {
        let name = name.into().trim().to_string();
        if name.is_empty() {
            return Err(DomainError::Required { field: "name" });
        }
        Ok(Self {
            name,
            quantity: normalize_quantity(quantity),
            category: normalize_optional(category),
            expiry_date,
        })
    }
}

fn normalize_quantity(quantity: Option<String>) -> String {
    quantity
        .map(|q| q.trim().to_string())
        .filter(|q| !q.is_empty())
        .unwrap_or_else(|| "1".to_string())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_item_rejects_blank_name() {
        assert_eq!(
            NewItem::new("   ", None, None, None).unwrap_err(),
            DomainError::Required { field: "name" }
        );
    }

    #[test]
    fn new_item_defaults_quantity_and_trims() {
        let item = NewItem::new("  Milk  ", Some("  ".into()), Some("".into()), None).unwrap();
        assert_eq!(item.name, "Milk");
        assert_eq!(item.quantity, "1");
        assert_eq!(item.category, None);
    }

    #[test]
    fn status_roundtrips_through_string() {
        for s in [ItemStatus::InFridge, ItemStatus::Used, ItemStatus::Tossed] {
            assert_eq!(s.as_str().parse::<ItemStatus>().unwrap(), s);
        }
    }
}
