/// deepmail-sandbox-file: static file analysis service.
///
/// Architecture:
///   - gRPC server (FileSandbox)  → on-demand analysis via AnalyzeFile/GetReport
///   - NATS consumer              → event-driven from parser pipeline
///   - Tools: file, exiftool, strings, binwalk, pdfid, oletools, pefile, yara-x
///   - Entropy: pure-Rust Shannon entropy
///   - Scorer: additive threat scoring with verdict classification

mod config;
mod consumer;
mod db;
mod entropy;
mod error;
mod pipeline;
mod rules;
mod s3;
mod scorer;
mod service;
mod tools;

use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tonic::transport::Server;

use deepmail_common::proto::sandbox_file::file_sandbox_server::FileSandboxServer;

use crate::config::SandboxFileConfig;
use crate::pipeline::JobCtx;
use crate::service::FileSandboxService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("deepmail-sandbox-file starting...");

    let cfg = SandboxFileConfig::from_env();

    // ── Database pools ──────────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;

    let parser_pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&cfg.parser_database_url)
        .await?;

    let ingest_pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&cfg.ingest_database_url)
        .await?;

    tracing::info!("database pools connected");

    // ── S3 client ───────────────────────────────────────────────────────
    let s3_config = aws_config::from_env()
        .endpoint_url(&cfg.s3_endpoint)
        .region(aws_config::Region::new(cfg.s3_region.clone()))
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&s3_config);

    // ── Compile YARA rules ──────────────────────────────────────────────
    let yara_rules = rules::compile_rules()
        .map_err(|e| anyhow::anyhow!("YARA compilation failed: {}", e))?;
    tracing::info!("YARA rules compiled");

    // ── NATS client ─────────────────────────────────────────────────────
    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("NATS connected");

    // ── Job context ─────────────────────────────────────────────────────
    let ctx = Arc::new(JobCtx {
        pool: Arc::new(pool),
        parser_pool: Arc::new(parser_pool),
        ingest_pool: Arc::new(ingest_pool),
        s3_client: Arc::new(s3_client),
        s3_bucket: cfg.s3_bucket.clone(),
        yara_rules: Arc::new(yara_rules),
        config: Arc::new(cfg.clone()),
        nats,
    });

    // ── Health service ──────────────────────────────────────────────────
    let (mut health_reporter, health_svc) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<FileSandboxServer<FileSandboxService>>()
        .await;

    // ── NATS consumer (background) ──────────────────────────────────────
    let consumer_ctx = Arc::clone(&ctx);
    tokio::spawn(async move {
        if let Err(e) = consumer::start_consumer(consumer_ctx).await {
            tracing::error!("NATS consumer failed: {}", e);
        }
    });

    // ── gRPC server ─────────────────────────────────────────────────────
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    let listener = TcpListener::bind(grpc_addr).await?;

    tracing::info!(%grpc_addr, "gRPC server listening");

    Server::builder()
        .add_service(health_svc)
        .add_service(FileSandboxServer::new(FileSandboxService {
            ctx: Arc::clone(&ctx),
        }))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
        .await?;

    Ok(())
}
