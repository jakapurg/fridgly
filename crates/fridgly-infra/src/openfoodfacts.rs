//! [`ProductCatalog`] adapter backed by the Open Food Facts public API.
//!
//! Open Food Facts is a free, crowd-sourced database of packaged foods keyed by
//! barcode. We query the read-only v2 product endpoint and map its (frequently
//! partial) records onto the domain [`ProductInfo`].

use std::time::Duration;

use async_trait::async_trait;
use fridgly_domain::{CatalogError, ProductCatalog, ProductInfo};
use serde::Deserialize;

/// Default base URL for the Open Food Facts product endpoint. The full request
/// is `{base}/{barcode}.json`.
const DEFAULT_BASE_URL: &str = "https://world.openfoodfacts.org/api/v2/product";

/// Open Food Facts asks API clients to send a descriptive `User-Agent`.
const USER_AGENT: &str = concat!(
    "Fridgly/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/extra-dev/fridgly)"
);

/// Product catalog that resolves barcodes via Open Food Facts.
pub struct OpenFoodFactsCatalog {
    client: reqwest::Client,
    base_url: String,
}

impl OpenFoodFactsCatalog {
    /// Build a catalog pointing at the public Open Food Facts API.
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL.to_string())
    }

    /// Build a catalog against a custom base URL (used in tests to point at a
    /// local stub server).
    pub fn with_base_url(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(8))
            .build()
            .expect("failed to build HTTP client");
        Self { client, base_url }
    }
}

impl Default for OpenFoodFactsCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Top-level shape of the v2 product response. `status` is `1` when a product
/// was found, `0` otherwise.
#[derive(Deserialize)]
struct OffResponse {
    #[serde(default)]
    status: i64,
    product: Option<OffProduct>,
}

/// The subset of product fields we request and use.
#[derive(Deserialize)]
struct OffProduct {
    product_name: Option<String>,
    brands: Option<String>,
    quantity: Option<String>,
    /// Comma-separated category path, most-general first.
    categories: Option<String>,
}

/// Trim a value and drop it if it ends up empty.
fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Pick the most-specific (last) label from a comma-separated category path.
fn last_category(categories: Option<String>) -> Option<String> {
    clean(categories).and_then(|c| {
        c.rsplit(',')
            .map(str::trim)
            .find(|s| !s.is_empty())
            .map(str::to_string)
    })
}

#[async_trait]
impl ProductCatalog for OpenFoodFactsCatalog {
    async fn lookup(&self, barcode: &str) -> Result<Option<ProductInfo>, CatalogError> {
        let code = barcode.trim();
        // Barcodes are numeric (EAN-8/13, UPC-A/E). Reject anything else up
        // front so we never hand junk to the upstream API.
        if code.is_empty() || code.len() > 20 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(CatalogError::InvalidBarcode);
        }

        let url = format!(
            "{}/{}.json?fields=product_name,brands,quantity,categories",
            self.base_url, code
        );
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CatalogError::Upstream(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CatalogError::Upstream(format!(
                "unexpected status {}",
                response.status()
            )));
        }

        let body: OffResponse = response
            .json()
            .await
            .map_err(|e| CatalogError::Upstream(e.to_string()))?;

        if body.status != 1 {
            return Ok(None);
        }
        let Some(product) = body.product else {
            return Ok(None);
        };

        // Fall back to the brand when no product name is recorded; without at
        // least a name there's nothing useful to pre-fill.
        let name = match clean(product.product_name).or_else(|| clean(product.brands)) {
            Some(name) => name,
            None => return Ok(None),
        };

        Ok(Some(ProductInfo {
            barcode: code.to_string(),
            name,
            quantity: clean(product.quantity),
            category: last_category(product.categories),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_non_numeric_barcode() {
        let catalog = OpenFoodFactsCatalog::new();
        assert!(matches!(
            catalog.lookup("abc123").await,
            Err(CatalogError::InvalidBarcode)
        ));
        assert!(matches!(
            catalog.lookup("   ").await,
            Err(CatalogError::InvalidBarcode)
        ));
    }

    #[test]
    fn last_category_takes_most_specific_label() {
        assert_eq!(
            last_category(Some("Dairies, Milks, Semi-skimmed milk".into())),
            Some("Semi-skimmed milk".to_string())
        );
        assert_eq!(last_category(Some("  ".into())), None);
        assert_eq!(last_category(None), None);
    }
}
