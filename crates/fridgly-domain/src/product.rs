//! The product-catalog *port*: resolving a packaged food's barcode into
//! descriptive details (name, pack size, category).
//!
//! Like [`crate::ItemRepository`], this is an interface the domain defines and
//! the infrastructure layer implements (e.g. against Open Food Facts). The web
//! layer depends only on this trait, so the upstream data source stays
//! swappable and can be faked in tests.

use async_trait::async_trait;

use crate::error::CatalogError;

/// Product details resolved from a barcode. Fields mirror what's needed to
/// pre-fill a [`crate::NewItem`]; everything except the name is optional because
/// external catalogs are frequently incomplete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductInfo {
    /// The EAN/UPC barcode this record was resolved from.
    pub barcode: String,
    /// Human-readable product name (e.g. `"Semi-skimmed milk"`).
    pub name: String,
    /// Pack size as printed on the label (e.g. `"1 L"`, `"500 g"`).
    pub quantity: Option<String>,
    /// A single, most-specific category label when the catalog provides one.
    pub category: Option<String>,
}

/// Read-only lookup of products by barcode.
///
/// Implementations must be `Send + Sync` so they can be shared across async
/// request handlers behind an `Arc`.
#[async_trait]
pub trait ProductCatalog: Send + Sync + 'static {
    /// Resolve a product by its (EAN/UPC) barcode.
    ///
    /// Returns `Ok(None)` when the barcode is well-formed but no matching
    /// product is known, and [`CatalogError::InvalidBarcode`] when the input
    /// isn't a plausible barcode at all.
    async fn lookup(&self, barcode: &str) -> Result<Option<ProductInfo>, CatalogError>;
}
