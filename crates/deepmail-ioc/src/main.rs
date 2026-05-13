mod cluster;
mod config;
mod consumer;
mod db;
mod enrich;
mod error;
mod extract;
mod normalize;
mod pipeline;
mod relations;
mod service;

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tonic::transport::{Channel, Server};

use deepmail_common::nats::create_jetstream;
use deepmail_common::proto::ioc::ioc_extractor_server::IocExtractorServer;

use crate::config::IocConfig;
use crate::enrich::IntelGrpcClient;
use crate::pipeline::PipelineCtx;
use crate::service::IocExtractorService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("deepmail-ioc starting");

    // a. Load config
    let cfg = IocConfig::load()
        .map_err(|e| anyhow::anyhow!("failed to load config: {e}"))?;

    // b. Connect main pool (deepmail_ioc)
    let main_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;
    tracing::info!("connected to deepmail_ioc database");

    // Run migrations
    sqlx::migrate!("../../migrations/deepmail-ioc")
        .run(&main_pool)
        .await?;
    tracing::info!("migrations applied");

    // c. Connect parser pool (cross-service)
    let parser_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.parser_database_url)
        .await?;
    tracing::info!("connected to parser database");

    // d. Connect ingest pool (cross-service)
    let ingest_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.ingest_database_url)
        .await?;
    tracing::info!("connected to ingest database");

    // e. Connect to deepmail-intel gRPC
    let intel_channel = Channel::from_shared(cfg.intel_grpc_url.clone())?
        .connect_lazy();
    let intel_client = Arc::new(IntelGrpcClient::new(intel_channel));
    tracing::info!(url = %cfg.intel_grpc_url, "intel gRPC client configured (lazy)");

    // f. Connect NATS JetStream
    let js = create_jetstream(&cfg.nats_url).await?;
    tracing::info!("NATS JetStream connected");

    // Ensure stream exists
    let stream = js
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "DEEPMAIL".to_string(),
            subjects: vec!["deepmail.>".to_string()],
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await?;

    // Build pipeline context
    let ctx = Arc::new(PipelineCtx {
        pool: main_pool.clone(),
        parser_pool,
        ingest_pool,
        intel_client,
        js: js.clone(),
        enrich_concurrency: cfg.enrich_concurrency,
        campaign_window_days: cfg.campaign_window_days,
        similarity_threshold: cfg.similarity_threshold,
    });

    // g. Create durable NATS consumer
    let nats_consumer = stream
        .get_or_create_consumer(
            "ioc-extractor",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("ioc-extractor".to_string()),
                filter_subject: "deepmail.jobs.analysis".to_string(),
                ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                max_deliver: 3,
                ack_wait: std::time::Duration::from_secs(120),
                ..Default::default()
            },
        )
        .await?;

    let consumer_ctx = ctx.clone();
    let consumer_handle = tokio::spawn(async move {
        consumer::run_consumer(nats_consumer, consumer_ctx).await;
    });

    // h. Start gRPC server with health check
    let grpc_addr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    let grpc_service = IocExtractorService::new(ctx.clone());

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<IocExtractorServer<IocExtractorService>>()
        .await;

    tracing::info!(addr = %grpc_addr, "gRPC server starting");

    let grpc_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(health_service)
            .add_service(IocExtractorServer::new(grpc_service))
            .serve(grpc_addr)
            .await
    });

    // i. Wait for shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("received ctrl-c, shutting down");
        }
        result = grpc_handle => {
            match result {
                Ok(Ok(())) => tracing::info!("gRPC server exited"),
                Ok(Err(e)) => tracing::error!(error = %e, "gRPC server error"),
                Err(e) => tracing::error!(error = %e, "gRPC task panicked"),
            }
        }
        _ = consumer_handle => {
            tracing::warn!("NATS consumer exited");
        }
    }

    tracing::info!("deepmail-ioc stopped");
    Ok(())
}
