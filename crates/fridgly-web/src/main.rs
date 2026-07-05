//! Fridgly web server entry point.
//!
//! Composition root: loads config, wires the Postgres adapter into the shared
//! state, runs migrations and serves the Axum app. All real logic lives in the
//! feature modules and the `fridgly-domain` / `fridgly-infra` crates.

mod app;
mod config;
mod error;
mod features;
mod i18n;
mod state;

use std::sync::Arc;

use anyhow::Context;
use fridgly_infra::PgItemRepository;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Config::from_env().context("loading configuration")?;

    let pool = fridgly_infra::connect(&config.database_url, config.db_max_connections)
        .await
        .context("connecting to Postgres")?;
    fridgly_infra::run_migrations(&pool)
        .await
        .context("running database migrations")?;
    tracing::info!("database ready, migrations applied");

    let repository = Arc::new(PgItemRepository::new(pool));
    let state = AppState::new(repository);
    let router = app::build_router(state, &config.static_dir);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("binding {}", config.bind_addr))?;
    tracing::info!("Fridgly listening on http://{}", config.bind_addr);

    axum::serve(listener, router)
        .await
        .context("running HTTP server")?;
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,fridgly_web=debug".into()),
        )
        .init();
}
