//! Router assembly: wires feature routes, static assets and middleware onto the
//! shared [`AppState`].

use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::features;
use crate::state::AppState;

/// Build the top-level application router.
pub fn build_router(state: AppState, static_dir: &str) -> Router {
    let router = Router::new()
        .merge(features::items::routes())
        .merge(features::meals::routes())
        .merge(features::pages::routes())
        .merge(features::products::routes())
        .nest_service("/static", ServeDir::new(static_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    with_livereload(router)
}

/// In dev builds, inject browser live-reload. The reload script is only added to
/// full-page navigations — htmx fragment responses (which carry the `HX-Request`
/// header) are left untouched so swaps aren't polluted with an extra script.
#[cfg(feature = "dev")]
fn with_livereload(router: Router) -> Router {
    use axum::http::Request;
    use tower_livereload::LiveReloadLayer;

    let layer = LiveReloadLayer::new().request_predicate(|req: &Request<axum::body::Body>| {
        !req.headers().contains_key("hx-request")
    });
    router.layer(layer)
}

#[cfg(not(feature = "dev"))]
fn with_livereload(router: Router) -> Router {
    router
}
