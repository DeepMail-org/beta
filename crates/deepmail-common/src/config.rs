//! Typed configuration loader.
//!
//! Layers, in increasing priority:
//! 1. `config.toml` in the working directory (optional).
//! 2. `config/{run_mode}.toml` where `run_mode` comes from `RUN_MODE` env
//!    (defaults to `development`).
//! 3. Environment variables prefixed with the supplied service prefix
//!    (e.g. `DEEPMAIL_AUTH_DATABASE_URL` → `database_url`).

use config::{Config, Environment, File};
use serde::de::DeserializeOwned;

/// Load typed configuration from config files and environment variables.
///
/// Environment variables override file values. The `prefix` parameter is the
/// env var prefix used for lookup; for example, `prefix = "DEEPMAIL_AUTH"`
/// reads `DEEPMAIL_AUTH_DATABASE_URL`, `DEEPMAIL_AUTH_GRPC__BIND_ADDR`, etc.
/// Double underscores (`__`) become nested-key separators, so
/// `DEEPMAIL_AUTH_GRPC__BIND_ADDR` maps to `grpc.bind_addr`.
pub fn load_config<T: DeserializeOwned>(prefix: &str) -> Result<T, config::ConfigError> {
    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

    let cfg = Config::builder()
        .add_source(File::with_name("config").required(false))
        .add_source(File::with_name(&format!("config/{run_mode}")).required(false))
        .add_source(
            Environment::with_prefix(prefix)
                .separator("__")
                .try_parsing(true),
        )
        .build()?;

    cfg.try_deserialize::<T>()
}
