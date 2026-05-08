/// deepmail-body: Email body content analysis service.
///
/// gRPC + NATS consumer for phishing detection, URL extraction,
/// QR code detection, HTML obfuscation analysis, and tracking pixel detection.

mod config;
mod consumer;
mod db;
mod error;
mod html;
mod keywords;
mod ml;
mod pipeline;
mod service;
mod urls;

use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::proto::body::body_analyzer_server::BodyAnalyzerServer;

use crate::config::BodyConfig;
use crate::ml::MlClient;
use crate::pipeline::PipelineCtx;
use crate::service::BodyAnalyzerService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-body starting");

    // ── a. Load config ─────────────────────────────────────────────────
    let cfg = BodyConfig::load().unwrap_or_else(|e| {
        // Fallback to env vars
        tracing::warn!("config load failed ({}), using env vars", e);
        BodyConfig {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_body".to_string()),
            parser_database_url: std::env::var("PARSER_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_parser".to_string()),
            ingest_database_url: std::env::var("INGEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ingest".to_string()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            grpc_port: std::env::var("GRPC_PORT")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(50059),
            ml_service_url: std::env::var("ML_SERVICE_URL").unwrap_or_default(),
            http_timeout_secs: std::env::var("HTTP_TIMEOUT_SECS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(5),
        }
    });

    // ── b. Connect main pool ───────────────────────────────────────────
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(15)
            .connect(&cfg.database_url)
            .await?,
    );
    tracing::info!("connected to deepmail_body DB");

    // Run migrations
    sqlx::migrate!("../../migrations/deepmail-body")
        .run(pool.as_ref())
        .await?;
    tracing::info!("migrations applied");

    // ── c. Connect parser pool ─────────────────────────────────────────
    let parser_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&cfg.parser_database_url)
            .await?,
    );
    tracing::info!("connected to parser DB");

    // ── d. Connect ingest pool ─────────────────────────────────────────
    let ingest_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&cfg.ingest_database_url)
            .await?,
    );
    tracing::info!("connected to ingest DB");

    // ── e. Build ML client if configured ───────────────────────────────
    let ml_client = if cfg.ml_service_url.is_empty() {
        tracing::info!("ML_SERVICE_URL not set, keyword scoring only");
        None
    } else {
        tracing::info!("ML client configured: {}", cfg.ml_service_url);
        Some(Arc::new(MlClient::new(cfg.ml_service_url.clone(), cfg.http_timeout_secs)))
    };

    // ── f. Connect NATS ────────────────────────────────────────────────
    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("connected to NATS");

    // Pipeline context
    let ctx = Arc::new(PipelineCtx {
        pool,
        parser_pool,
        ingest_pool,
        ml_client,
        nats,
    });

    // ── g. Start NATS consumer ─────────────────────────────────────────
    let consumer_ctx = Arc::clone(&ctx);
    tokio::spawn(async move {
        consumer::run_consumer(consumer_ctx).await;
    });

    // ── h. Start gRPC server ───────────────────────────────────────────
    let addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%addr, "starting gRPC server");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<BodyAnalyzerServer<BodyAnalyzerService>>()
        .await;

    let svc = BodyAnalyzerService { ctx };

    // ── i. Serve + graceful shutdown ───────────────────────────────────
    Server::builder()
        .add_service(health_service)
        .add_service(BodyAnalyzerServer::new(svc))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("shutting down");
        })
        .await?;

    Ok(())
}
