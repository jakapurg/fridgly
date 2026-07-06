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

/// A count of individual pieces remaining inside a container item, e.g. the
/// "3 eggs" left in a packet. Tracked separately from [`Item::quantity`], which
/// counts the containers themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Subunit {
    /// How many pieces are left. Never negative.
    pub remaining: i32,
    /// What the pieces are (e.g. `"eggs"`), if named.
    pub unit: Option<String>,
}

impl Subunit {
    /// Build a subunit from an optional remaining count and unit label.
    ///
    /// Returns `None` when no remaining count is supplied (there is nothing to
    /// track); a negative count is clamped to zero.
    pub fn new(remaining: Option<i32>, unit: Option<String>) -> Option<Self> {
        remaining.map(|remaining| Self {
            remaining: remaining.max(0),
            unit: normalize_optional(unit),
        })
    }
}

/// A persisted fridge item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub quantity: String,
    /// Outer unit paired with `quantity`, e.g. `"packet"` in "1 packet".
    pub unit: Option<String>,
    /// Individual pieces remaining inside the container, if tracked.
    pub subunit: Option<Subunit>,
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
    pub unit: Option<String>,
    pub subunit: Option<Subunit>,
    pub category: Option<String>,
    pub expiry_date: Option<NaiveDate>,
}

impl NewItem {
    /// Build a validated `NewItem`, normalising whitespace and applying the
    /// default quantity of `"1"` when none is supplied.
    pub fn new(
        name: impl Into<String>,
        quantity: Option<String>,
        unit: Option<String>,
        subunit: Option<Subunit>,
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
            unit: normalize_optional(unit),
            subunit,
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
    pub unit: Option<String>,
    pub subunit: Option<Subunit>,
    pub category: Option<String>,
    pub expiry_date: Option<NaiveDate>,
}

impl ItemChanges {
    pub fn new(
        name: impl Into<String>,
        quantity: Option<String>,
        unit: Option<String>,
        subunit: Option<Subunit>,
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
            unit: normalize_optional(unit),
            subunit,
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
            NewItem::new("   ", None, None, None, None, None).unwrap_err(),
            DomainError::Required { field: "name" }
        );
    }

    #[test]
    fn new_item_defaults_quantity_and_trims() {
        let item = NewItem::new(
            "  Milk  ",
            Some("  ".into()),
            Some("  ".into()),
            None,
            Some("".into()),
            None,
        )
        .unwrap();
        assert_eq!(item.name, "Milk");
        assert_eq!(item.quantity, "1");
        assert_eq!(item.unit, None);
        assert_eq!(item.subunit, None);
        assert_eq!(item.category, None);
    }

    #[test]
    fn new_item_carries_container_and_subunit() {
        let subunit = Subunit::new(Some(3), Some("  eggs  ".into()));
        let item = NewItem::new(
            "Eggs",
            Some("1".into()),
            Some(" packet ".into()),
            subunit,
            None,
            None,
        )
        .unwrap();
        assert_eq!(item.unit.as_deref(), Some("packet"));
        let subunit = item.subunit.expect("subunit present");
        assert_eq!(subunit.remaining, 3);
        assert_eq!(subunit.unit.as_deref(), Some("eggs"));
    }

    #[test]
    fn subunit_requires_a_count_and_clamps_negatives() {
        assert!(Subunit::new(None, Some("eggs".into())).is_none());
        let clamped = Subunit::new(Some(-4), None).expect("count present");
        assert_eq!(clamped.remaining, 0);
        assert_eq!(clamped.unit, None);
    }

    #[test]
    fn status_roundtrips_through_string() {
        for s in [ItemStatus::InFridge, ItemStatus::Used, ItemStatus::Tossed] {
            assert_eq!(s.as_str().parse::<ItemStatus>().unwrap(), s);
        }
    }
}
