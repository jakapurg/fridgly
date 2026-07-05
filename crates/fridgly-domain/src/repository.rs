//! The storage *port*: an interface the domain defines and the infrastructure
//! layer implements. The web layer depends only on this trait, never on a
//! concrete database type, which keeps persistence swappable and testable.

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::RepositoryError;
use crate::item::{Item, ItemChanges, ItemStatus, NewItem};

type Result<T> = std::result::Result<T, RepositoryError>;

/// Persistence operations for fridge items.
///
/// Implementations must be `Send + Sync` so they can be shared across async
/// request handlers behind an `Arc`.
#[async_trait]
pub trait ItemRepository: Send + Sync + 'static {
    /// All items currently in the fridge, ordered most-urgent first
    /// (soonest expiry first; items without a date last).
    async fn list_in_fridge(&self) -> Result<Vec<Item>>;

    /// In-fridge items whose name matches `query` (case-insensitive substring),
    /// in the same ordering as [`list_in_fridge`]. An empty/whitespace query
    /// returns everything.
    async fn search_in_fridge(&self, query: &str) -> Result<Vec<Item>>;

    /// Fetch a single item by id, regardless of status.
    async fn find(&self, id: Uuid) -> Result<Option<Item>>;

    /// Insert a new item and return the stored row.
    async fn create(&self, new_item: NewItem) -> Result<Item>;

    /// Apply edits to an existing in-fridge item.
    ///
    /// Returns [`RepositoryError::NotFound`] if no matching in-fridge item
    /// exists.
    async fn update(&self, id: Uuid, changes: ItemChanges) -> Result<Item>;

    /// Move an item to a new lifecycle [`ItemStatus`] (used / tossed / …).
    async fn set_status(&self, id: Uuid, status: ItemStatus) -> Result<()>;

    /// Permanently remove an item.
    async fn delete(&self, id: Uuid) -> Result<()>;
}
