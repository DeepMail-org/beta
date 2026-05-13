/// deepmail-sandbox-url: URL behavioral sandbox service.
///
/// gRPC + NATS consumer for Docker-based URL sandboxing with
/// Playwright browser automation, QR code decoding, and threat classification.

mod classifier;
mod config;
mod consumer;
mod db;
mod docker;
mod error;
mod pipeline;
mod playwright_script;
mod qr;
mod s3;
mod service;

use std::net::SocketAddr;
use std::sync::Arc;

use bollard::Docker;
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::proto::sandbox_url::url_sandbox_server::UrlSandboxServer;

use crate::config::SandboxUrlConfig;
use crate::pipeline::JobCtx;
use crate::service::UrlSandboxService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-sandbox-url starting");

    // ── a. Load config ─────────────────────────────────────────────────
    let cfg = SandboxUrlConfig::load().unwrap_or_else(|e| {
        tracing::warn!("config load failed ({}), using env vars", e);
        SandboxUrlConfig::from_env()
    });
    let cfg = Arc::new(cfg);

    // ── b. Connect DB pool ─────────────────────────────────────────────
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(15)
            .connect(&cfg.database_url)
            .await?,
    );
    tracing::info!("connected to deepmail_sandbox_url DB");

    // Run migrations
    sqlx::migrate!("../../migrations/deepmail-sandbox-url")
        .run(pool.as_ref())
        .await?;
    tracing::info!("migrations applied");

    // ── c. Connect Docker ──────────────────────────────────────────────
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => {
            tracing::info!("connected to Docker");
            Arc::new(d)
        }
        Err(e) => {
            tracing::error!("Docker connection failed: {}", e);
            let fallback = Docker::connect_with_socket(
                &cfg.docker_sock,
                120,
                bollard::API_DEFAULT_VERSION,
            )
            .map_err(|err| anyhow::anyhow!("cannot connect to Docker: {err}"))?;
            Arc::new(fallback)
        }
    };

    // ── d. Build S3 client ─────────────────────────────────────────────
    let s3_config = aws_config::from_env()
        .endpoint_url(&cfg.s3_endpoint)
        .region(aws_config::Region::new(cfg.s3_region.clone()))
        .load()
        .await;

    let s3_client = Arc::new(
        aws_sdk_s3::Client::from_conf(
            aws_sdk_s3::config::Builder::from(&s3_config)
                .force_path_style(true)
                .build(),
        ),
    );
    tracing::info!("S3 client configured (endpoint: {})", cfg.s3_endpoint);

    // ── e. Connect NATS ────────────────────────────────────────────────
    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("connected to NATS");

    // ── Build shared context ───────────────────────────────────────────
    let ctx = Arc::new(JobCtx {
        pool,
        docker,
        s3_client,
        s3_bucket: cfg.s3_bucket.clone(),
        config: Arc::clone(&cfg),
        nats,
    });

    // ── f. Start NATS consumer ─────────────────────────────────────────
    let consumer_ctx = Arc::clone(&ctx);
    tokio::spawn(async move {
        consumer::run_consumer(consumer_ctx).await;
    });

    // ── g. Start gRPC server ───────────────────────────────────────────
    let addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%addr, "starting gRPC server");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<UrlSandboxServer<UrlSandboxService>>()
        .await;

    let svc = UrlSandboxService { ctx };

    // ── h. Serve with shutdown ─────────────────────────────────────────
    Server::builder()
        .add_service(health_service)
        .add_service(UrlSandboxServer::new(svc))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("shutting down");
        })
        .await?;

    Ok(())
}
