//! Barcode → product lookup endpoint backing the scan-to-add flow.
//!
//! The scanner in the browser reads a barcode, then calls this endpoint to
//! resolve it into product details (via the [`ProductCatalog`] port) which the
//! front-end uses to pre-fill the add-item sheet.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use fridgly_domain::{CatalogError, ProductInfo};
use serde::Serialize;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/products/:barcode", get(lookup))
}

/// JSON payload consumed by the scanner front-end.
#[derive(Serialize)]
struct ProductResponse {
    barcode: String,
    name: String,
    quantity: Option<String>,
    category: Option<String>,
}

impl From<ProductInfo> for ProductResponse {
    fn from(p: ProductInfo) -> Self {
        Self {
            barcode: p.barcode,
            name: p.name,
            quantity: p.quantity,
            category: p.category,
        }
    }
}

/// Resolve a scanned barcode. Returns the product as JSON, `404` when the
/// barcode is unknown, `400` when it isn't a plausible barcode, and `502` when
/// the upstream catalog is unavailable.
async fn lookup(State(state): State<AppState>, Path(barcode): Path<String>) -> Response {
    match state.catalog().lookup(&barcode).await {
        Ok(Some(product)) => Json(ProductResponse::from(product)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "product not found").into_response(),
        Err(CatalogError::InvalidBarcode) => {
            (StatusCode::BAD_REQUEST, "invalid barcode").into_response()
        }
        Err(CatalogError::Upstream(msg)) => {
            tracing::warn!(error = %msg, "product catalog lookup failed");
            (StatusCode::BAD_GATEWAY, "product lookup failed").into_response()
        }
    }
}
