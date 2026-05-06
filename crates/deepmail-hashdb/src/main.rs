//! deepmail-hashdb — File Hash Registry Service
//!
//! gRPC server only. No HTTP, no NATS consumer.
//! Provides bloom filter pre-check + PostgreSQL exact lookup
//! for file deduplication and verdict caching.

use std::sync::Arc;

use deepmail_common::proto::hashdb::hash_db_service_server::HashDbServiceServer;
use tonic::transport::Server;
use tracing::info;

mod bloom;
mod config;
mod db;
mod error;
mod fuzzy;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Tracing ──────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "deepmail_hashdb=info,deepmail_common=info".into()
                }),
        )
        .with_current_span(true)
        .init();

    info!(service = "deepmail-hashdb", "starting");

    // ── Config ───────────────────────────────────────────────────
    let config = config::Config::load()
        .map_err(|e| anyhow::anyhow!("config error: {e}"))?;
    let config = Arc::new(config);

    // ── Database ─────────────────────────────────────────────────
    let pool = deepmail_common::db::create_pool(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("db pool error: {e}"))?;
    let pool = Arc::new(pool);

    sqlx::migrate!("../../migrations/deepmail-hashdb")
        .run(pool.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("migration error: {e}"))?;
    info!("database migrations applied");

    // ── Redis ────────────────────────────────────────────────────
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .map_err(|e| anyhow::anyhow!("redis client error: {e}"))?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .map_err(|e| anyhow::anyhow!("redis connection error: {e}"))?;
    let redis = Arc::new(tokio::sync::Mutex::new(redis_conn));
    info!(url = %config.redis_url, "Redis connected");

    // ── gRPC server ──────────────────────────────────────────────
    let addr = config
        .grpc_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid grpc_addr: {e}"))?;

    let hashdb_service = service::HashDbServiceImpl {
        pool,
        redis,
        config,
    };

    info!(addr = %addr, "deepmail-hashdb gRPC server listening");

    Server::builder()
        .add_service(HashDbServiceServer::new(hashdb_service))
        .serve_with_shutdown(addr, async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to listen for ctrl-c");
                std::future::pending::<()>().await;
            }
            info!("shutdown signal received — draining");
        })
        .await
        .map_err(|e| anyhow::anyhow!("gRPC server error: {e}"))?;

    info!("deepmail-hashdb stopped");
    Ok(())
}
