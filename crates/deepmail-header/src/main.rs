//! deepmail-header entry point.
//!
//! Bootstraps:
//!   1. Three PostgreSQL pools (header, ingest, parser)
//!   2. Hickory DNS resolver (system or custom)
//!   3. NATS client + JetStream consumer
//!   4. gRPC server on the configured port
//!
//! All three subsystems (gRPC, NATS consumer) run concurrently via
//! `tokio::select!` with graceful shutdown on SIGINT / SIGTERM.

mod checks;
mod config;
mod consumer;
mod dkim;
mod dmarc;
mod dns;
mod error;
mod pipeline;
mod service;
mod spf;

use std::sync::Arc;

use deepmail_common::db::create_pool;
use deepmail_common::proto::header::header_analyzer_server::HeaderAnalyzerServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── Logging ───────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,deepmail_header=debug".parse().unwrap()),
        )
        .json()
        .init();

    tracing::info!("deepmail-header starting");

    // ── Config ────────────────────────────────────────────────────
    let cfg = config::Config::load()?;

    // ── Database pools ────────────────────────────────────────────
    let header_pool = create_pool(&cfg.database_url).await?;
    let ingest_pool = create_pool(&cfg.ingest_database_url).await?;
    let parser_pool = create_pool(&cfg.parser_database_url).await?;

    // Run migrations on header DB only (this service owns it)
    sqlx::migrate!("../../migrations/deepmail-header")
        .run(&header_pool)
        .await?;

    tracing::info!("database pools ready, migrations applied");

    // ── DNS resolver ──────────────────────────────────────────────
    let resolver = if cfg.dns_resolver.is_empty() {
        dns::Resolver::new_system()?
    } else {
        dns::Resolver::new_with_server(&cfg.dns_resolver)?
    };
    let resolver = Arc::new(resolver);

    // ── NATS client ───────────────────────────────────────────────
    let nats_client = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!(url = %cfg.nats_url, "NATS connected");

    // ── gRPC server ───────────────────────────────────────────────
    let grpc_addr = cfg.grpc_addr.parse()?;
    let header_service = service::HeaderService {
        header_pool: header_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        parser_pool: parser_pool.clone(),
        resolver: resolver.clone(),
    };

    let grpc_server = Server::builder()
        .add_service(HeaderAnalyzerServer::new(header_service))
        .serve(grpc_addr);

    // ── NATS consumer ─────────────────────────────────────────────
    let consumer_handle = consumer::run(
        nats_client,
        header_pool.clone(),
        ingest_pool.clone(),
        parser_pool.clone(),
        resolver.clone(),
        cfg.concurrency,
    );

    tracing::info!(grpc_addr = %cfg.grpc_addr, "deepmail-header ready");

    // ── Run concurrently with graceful shutdown ───────────────────
    tokio::select! {
        result = grpc_server => {
            if let Err(e) = result {
                tracing::error!(error = %e, "gRPC server error");
            }
        }
        result = consumer_handle => {
            if let Err(e) = result {
                tracing::error!(error = %e, "NATS consumer error");
            }
        }
        _ = async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to listen for ctrl+c");
            }
        } => {
            tracing::info!("shutdown signal received");
        }
    }

    tracing::info!("deepmail-header stopped");
    Ok(())
}
