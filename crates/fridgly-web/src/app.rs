//! Router assembly: wires feature routes, static assets and middleware onto the
//! shared [`AppState`].

use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::features;
use crate::state::AppState;

/// Build the top-level application router.
pub fn build_router(state: AppState, static_dir: &str) -> Router {
    Router::new()
        .merge(features::items::routes())
        .merge(features::pages::routes())
        .nest_service("/static", ServeDir::new(static_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
