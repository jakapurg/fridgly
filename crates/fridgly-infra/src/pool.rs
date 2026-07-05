//! Database connection pool and migration runner.

use sqlx::postgres::PgPoolOptions;
pub use sqlx::PgPool;

/// Open a connection pool to the given Postgres URL.
pub async fn connect(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}

/// Apply all pending schema migrations.
///
/// Migrations live at the workspace root in `migrations/` and are embedded into
/// the binary at compile time, so deployments need only the binary.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}
