mod cape;
mod config;
mod consumer;
mod db;
mod error;
mod fallback;
mod parser;
mod pipeline;
mod s3;
mod scorer;
mod service;

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use deepmail_common::proto::sandbox_dynamic::dynamic_sandbox_server::DynamicSandboxServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-sandbox-dynamic starting");

    // ── a. Load config ─────────────────────────────────────────────────
    let config = Arc::new(config::DynamicConfig::from_env());

    // ── b. Main DB pool (sandbox_dynamic) ──────────────────────────────
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await?,
    );

    // ── c. Static pool (sandbox_file — for fallback) ───────────────────
    let static_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(3)
            .connect(&config.sandbox_file_database_url)
            .await?,
    );

    // ── d. Ingest pool (cross-service progress) ────────────────────────
    let ingest_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(3)
            .connect(&config.ingest_database_url)
            .await?,
    );

    // ── e. S3 client ───────────────────────────────────────────────────
    let s3_config = aws_config::from_env()
        .endpoint_url(&config.s3_endpoint)
        .region(aws_sdk_s3::config::Region::new(config.s3_region.clone()))
        .load()
        .await;
    let s3_client = Arc::new(aws_sdk_s3::Client::new(&s3_config));

    // ── f. CAPEv2 client ───────────────────────────────────────────────
    let cape = Arc::new(cape::CapeClient::new(
        config.cape_api_url.clone(),
        config.cape_api_token.clone(),
    )?);

    if cape.is_configured() {
        tracing::info!(url = %config.cape_api_url, "CAPEv2 configured");
    } else {
        tracing::warn!("CAPEv2 not configured — fallback to static analysis");
    }

    // ── g. NATS ────────────────────────────────────────────────────────
    let nats = async_nats::connect(&config.nats_url).await?;
    tracing::info!(url = %config.nats_url, "NATS connected");

    // ── Build shared context ───────────────────────────────────────────
    let ctx = Arc::new(pipeline::JobCtx {
        pool: Arc::clone(&pool),
        static_pool,
        ingest_pool,
        cape,
        s3_client,
        s3_bucket: config.s3_bucket.clone(),
        config: Arc::clone(&config),
        nats,
    });

    // ── h. Start NATS consumer ─────────────────────────────────────────
    let consumer_ctx = Arc::clone(&ctx);
    tokio::spawn(async move {
        if let Err(e) = consumer::start_consumer(consumer_ctx).await {
            tracing::error!("NATS consumer error: {}", e);
        }
    });

    // ── i. gRPC server with health ─────────────────────────────────────
    let addr = format!("0.0.0.0:{}", config.grpc_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "gRPC listening");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<DynamicSandboxServer<service::DynamicSandboxService>>()
        .await;

    let svc = service::DynamicSandboxService { ctx };

    // ── j. Serve ───────────────────────────────────────────────────────
    let grpc_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(health_service)
            .add_service(DynamicSandboxServer::new(svc))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
    });

    tokio::select! {
        res = grpc_handle => {
            if let Ok(Err(e)) = res {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutting down");
        }
    }

    Ok(())
}
