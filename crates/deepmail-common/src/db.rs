use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::error::DeepMailError;

/// Creates a PostgreSQL connection pool with production-grade settings.
///
/// # Arguments
/// * `database_url` — Full PostgreSQL connection string
///   (e.g. `postgres://user:pass@host:5432/dbname`)
///
/// # Errors
/// Returns `DeepMailError::Database` if the pool cannot be established.
#[tracing::instrument(skip(database_url))]
pub async fn create_pg_pool(database_url: &str) -> Result<PgPool, DeepMailError> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .after_connect(|_conn, _meta| {
            Box::pin(async move {
                tracing::debug!("new database connection established");
                Ok(())
            })
        })
        .connect(database_url)
        .await?;

    tracing::info!("PostgreSQL connection pool created");
    Ok(pool)
}

/// Runs all pending migrations from the given directory.
///
/// Uses SQLx's embedded migration support. Migrations must be
/// idempotent (IF NOT EXISTS on all DDL).
#[tracing::instrument(skip(pool))]
pub async fn run_migrations(pool: &PgPool, migrator: &sqlx::migrate::Migrator) -> Result<(), DeepMailError> {
    migrator
        .run(pool)
        .await
        .map_err(|e| DeepMailError::Database(e.into()))?;

    tracing::info!("database migrations applied successfully");
    Ok(())
}
