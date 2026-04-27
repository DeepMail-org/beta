use crate::error::DeepMailError;

/// Reads a required environment variable, returning a typed `DeepMailError`
/// if the variable is missing or empty.
pub fn require_env(key: &str) -> Result<String, DeepMailError> {
    std::env::var(key).map_err(|_| {
        DeepMailError::Config(format!("required environment variable {key} is not set"))
    })
}

/// Reads an optional environment variable, returning `None` if unset.
pub fn optional_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

/// Reads an environment variable and parses it as the given type.
/// Returns the provided default if the variable is unset.
pub fn env_or_default<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(key) {
        Ok(val) => val.parse::<T>().unwrap_or_else(|e| {
            tracing::warn!(
                key,
                error = %e,
                "failed to parse env var, using default"
            );
            default
        }),
        Err(_) => default,
    }
}

/// Standard service configuration shared by all microservices.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// gRPC listen address (e.g. `0.0.0.0:50051`)
    pub grpc_addr: String,
    /// PostgreSQL connection URL
    pub database_url: String,
    /// NATS server URL
    pub nats_url: String,
    /// Service name for tracing
    pub service_name: String,
    /// Log level filter (e.g. `info`, `debug,sqlx=warn`)
    pub log_level: String,
}

impl ServiceConfig {
    /// Loads standard service configuration from environment variables.
    ///
    /// Required env vars:
    /// - `DATABASE_URL`
    /// - `NATS_URL`
    ///
    /// Optional env vars (with defaults):
    /// - `GRPC_ADDR` → `0.0.0.0:50051`
    /// - `SERVICE_NAME` → the provided `default_name`
    /// - `LOG_LEVEL` → `info`
    pub fn from_env(default_name: &str) -> Result<Self, DeepMailError> {
        Ok(Self {
            grpc_addr: env_or_default("GRPC_ADDR", "0.0.0.0:50051".to_string()),
            database_url: require_env("DATABASE_URL")?,
            nats_url: require_env("NATS_URL")?,
            service_name: env_or_default("SERVICE_NAME", default_name.to_string()),
            log_level: env_or_default("LOG_LEVEL", "info".to_string()),
        })
    }
}

/// Initializes the tracing subscriber with JSON-formatted logs.
///
/// Must be called once at service startup before any other tracing calls.
pub fn init_tracing(service_name: &str, log_level: &str) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    fmt()
        .json()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_current_span(true)
        .init();

    tracing::info!(service = service_name, "tracing initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_or_default_returns_default_when_unset() {
        let val: u16 = env_or_default("__DEEPMAIL_TEST_NONEXISTENT__", 8080);
        assert_eq!(val, 8080);
    }

    #[test]
    fn require_env_errors_on_missing() {
        let result = require_env("__DEEPMAIL_TEST_NONEXISTENT__");
        assert!(result.is_err());
    }
}
