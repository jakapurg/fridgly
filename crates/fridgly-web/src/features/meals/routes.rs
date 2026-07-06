//! HTTP routes for the meals feature.

use axum::routing::{get, post};
use axum::Router;

use super::handlers;
use crate::state::AppState;

/// All meal routes, to be merged into the application router.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/meals", get(handlers::page))
        .route("/meals/suggest", post(handlers::suggest))
}
