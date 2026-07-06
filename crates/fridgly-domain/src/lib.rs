//! Fridgly domain layer.
//!
//! This crate holds the business model and rules with **no dependency on any
//! web framework or database driver**. It defines:
//!
//! * the core [`item`] types ([`Item`], [`NewItem`], [`ItemChanges`], …),
//! * the expiry/urgency rules that decide how close food is to going off, and
//! * the [`ItemRepository`] *port* — a trait the persistence layer implements.
//!
//! Keeping this layer framework-free means the rules can be unit-tested in
//! isolation and the storage/transport technology can change without touching
//! business logic.

pub mod error;
pub mod item;
pub mod locale;
pub mod meals;
pub mod product;
pub mod repository;

pub use error::{CatalogError, DomainError, RepositoryError};
pub use item::{Item, ItemChanges, ItemStatus, ItemView, NewItem, Subunit, Urgency};
pub use locale::Locale;
pub use meals::{MealError, MealSuggester, MealSuggestion, MealType};
pub use product::{ProductCatalog, ProductInfo};
pub use repository::ItemRepository;
