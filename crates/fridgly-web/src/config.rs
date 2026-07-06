//! Application configuration, loaded from the environment.

use std::env::VarError;

/// Runtime configuration. Populated from environment variables (a local
/// `.env` file is loaded first in development).
#[derive(Debug, Clone)]
pub struct Config {
    /// Postgres connection string.
    pub database_url: String,
    /// Socket address to bind the HTTP server to.
    pub bind_addr: String,
    /// Maximum size of the database connection pool.
    pub db_max_connections: u32,
    /// Directory served at `/static`.
    pub static_dir: String,
    /// Anthropic API key for AI meal suggestions. When absent, meal
    /// suggestions are disabled (the screen shows a "not set up" message).
    pub anthropic_api_key: Option<String>,
    /// Claude model used for meal suggestions.
    pub anthropic_model: String,
}

impl Config {
    /// Read configuration from the environment.
    ///
    /// `DATABASE_URL` is required; everything else has a sensible default.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: require("DATABASE_URL")?,
            bind_addr: optional("BIND_ADDR", "0.0.0.0:3000"),
            db_max_connections: optional("DB_MAX_CONNECTIONS", "5")
                .parse()
                .map_err(|_| ConfigError::Invalid("DB_MAX_CONNECTIONS"))?,
            static_dir: optional("STATIC_DIR", "crates/fridgly-web/static"),
            anthropic_api_key: optional_var("ANTHROPIC_API_KEY"),
            anthropic_model: optional("ANTHROPIC_MODEL", "claude-opus-4-8"),
        })
    }
}

/// Read an optional variable, returning `None` when unset or empty.
fn optional_var(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

fn require(key: &'static str) -> Result<String, ConfigError> {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => Ok(v),
        Ok(_) | Err(VarError::NotPresent) => Err(ConfigError::Missing(key)),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::Invalid(key)),
    }
}

fn optional(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("required environment variable {0} is not set")]
    Missing(&'static str),
    #[error("environment variable {0} has an invalid value")]
    Invalid(&'static str),
}
