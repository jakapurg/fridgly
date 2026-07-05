//! Fridgly infrastructure layer.
//!
//! Concrete adapters that fulfil the ports defined in `fridgly-domain`, plus
//! the database connection pool and schema migrations. This is the only crate
//! that knows about `sqlx`/Postgres.

mod pool;
mod postgres;

pub use pool::{connect, run_migrations, PgPool};
pub use postgres::PgItemRepository;
