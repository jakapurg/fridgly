//! HTTP routes for the items feature.

use axum::routing::{get, post, put};
use axum::Router;

use super::handlers;
use crate::state::AppState;

/// All item routes, to be merged into the application router.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::index))
        .route("/items", post(handlers::create))
        .route("/items/:id", put(handlers::update).delete(handlers::delete))
        .route("/items/:id/used", post(handlers::mark_used))
        .route("/items/:id/tossed", post(handlers::mark_tossed))
        .route("/items/:id/edit", get(handlers::edit_form))
        .route("/items/:id/row", get(handlers::row))
}
