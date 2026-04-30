//! PostgreSQL connection pool helpers.
//!
//! All DeepMail services share a single `PgPool` per process configured with
//! production-grade defaults. Tune via env / config when a service has
//! markedly different connection pressure.

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::DeepMailError;

/// Maximum simultaneous connections held by the pool.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 20;

/// Minimum idle connections kept alive at all times.
pub const DEFAULT_MIN_CONNECTIONS: u32 = 2;

/// How long [`create_pool`] will wait for a connection before failing.
pub const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);

/// Idle connections are reaped after this duration.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(600);

/// Hard upper bound on a single connection's lifetime.
pub const DEFAULT_MAX_LIFETIME: Duration = Duration::from_secs(1800);

/// Create a PostgreSQL connection pool with production-grade settings.
///
/// Defaults:
/// - `max_connections`: 20
/// - `min_connections`: 2
/// - `acquire_timeout`: 30s
/// - `idle_timeout`: 600s
/// - `max_lifetime`: 1800s
/// - `test_before_acquire`: true (rejects half-open sockets)
///
/// `database_url` must be a libpq-style URI, e.g.
/// `postgres://user:pass@host:5432/dbname?sslmode=require`.
pub async fn create_pool(database_url: &str) -> Result<PgPool, DeepMailError> {
    let pool = PgPoolOptions::new()
        .max_connections(DEFAULT_MAX_CONNECTIONS)
        .min_connections(DEFAULT_MIN_CONNECTIONS)
        .acquire_timeout(DEFAULT_ACQUIRE_TIMEOUT)
        .idle_timeout(Some(DEFAULT_IDLE_TIMEOUT))
        .max_lifetime(Some(DEFAULT_MAX_LIFETIME))
        .test_before_acquire(true)
        .connect(database_url)
        .await?;

    tracing::info!(
        max_connections = DEFAULT_MAX_CONNECTIONS,
        min_connections = DEFAULT_MIN_CONNECTIONS,
        "postgres pool established"
    );

    Ok(pool)
}

/// Cheap health-check: round-trips a `SELECT 1` to verify the pool is live.
pub async fn ping(pool: &PgPool) -> Result<(), DeepMailError> {
    sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool).await?;
    Ok(())
}
