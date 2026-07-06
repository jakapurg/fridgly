//! Domain-level error types.

use thiserror::Error;

/// Errors that a [`crate::ItemRepository`] implementation may return.
///
/// The variants are storage-agnostic on purpose: adapters map their own
/// technology-specific failures (e.g. a `sqlx::Error`) into these so the rest
/// of the application never depends on a particular database driver.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// The requested entity does not exist.
    #[error("item not found")]
    NotFound,

    /// Any other failure originating in the storage backend.
    #[error("storage error: {0}")]
    Backend(String),
}

/// Errors produced by domain rules / validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    /// A required field was empty or otherwise invalid.
    #[error("{field} is required")]
    Required { field: &'static str },
}

/// Errors that a [`crate::ProductCatalog`] implementation may return.
///
/// Like [`RepositoryError`], these are technology-agnostic: an adapter talking
/// to a specific food database maps its own HTTP/parse failures into these.
#[derive(Debug, Error)]
pub enum CatalogError {
    /// The supplied barcode was empty or not a plausible product code.
    #[error("invalid barcode")]
    InvalidBarcode,

    /// The upstream catalog was unreachable or returned an unexpected response.
    #[error("catalog error: {0}")]
    Upstream(String),
}
