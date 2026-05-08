mod cache;
mod circuit;
mod config;
mod consumer;
mod db;
mod enricher;
mod error;
mod pipeline;
mod providers;
mod service;
mod telemetry;

use std::sync::Arc;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::nats::create_jetstream;
use deepmail_common::proto::intel::intel_enricher_server::IntelEnricherServer;

use crate::circuit::CircuitRegistry;
use crate::config::IntelConfig;
use crate::enricher::EnrichCtx;
use crate::providers::abuseipdb::AbuseIpDbClient;
use crate::providers::greynoise::GreyNoiseClient;
use crate::providers::ipinfo::IpInfoClient;
use crate::providers::otx::OtxClient;
use crate::providers::shodan::ShodanClient;
use crate::providers::virustotal::VtClient;
use crate::service::IntelEnricherService;
use crate::telemetry::TelemetryAccumulator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("deepmail-intel starting");

    // a. Load config
    let cfg = IntelConfig::load().expect("failed to load config");

    // b. Connect main DB pool (deepmail_intel)
    let main_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;
    tracing::info!("connected to deepmail_intel database");

    // Run migrations
    sqlx::migrate!("../../migrations/deepmail-intel")
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

    // e. Connect Redis
    let redis_client = redis::Client::open(cfg.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;
    tracing::info!("connected to Redis");

    // f. Build HTTP client
    let http_client = Arc::new(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(cfg.http_timeout_secs))
            .build()?,
    );

    // g. Initialise circuit breakers
    let circuits = Arc::new(CircuitRegistry::new());

    // h. Initialise telemetry accumulator
    let telemetry = Arc::new(TelemetryAccumulator::new());

    // i. Build provider clients
    let vt_client = Arc::new(VtClient::new(
        http_client.clone(),
        cfg.virustotal_api_key.clone(),
        cfg.vt_rate_limit_per_min,
    ));
    let abuse_client = Arc::new(AbuseIpDbClient::new(
        http_client.clone(),
        cfg.abuseipdb_api_key.clone(),
    ));
    let greynoise_client = Arc::new(GreyNoiseClient::new(
        http_client.clone(),
        cfg.greynoise_api_key.clone(),
    ));
    let ipinfo_client = Arc::new(IpInfoClient::new(
        http_client.clone(),
        cfg.ipinfo_token.clone(),
    ));
    let shodan_client = Arc::new(ShodanClient::new(
        http_client.clone(),
        cfg.shodan_api_key.clone(),
    ));
    let otx_client = Arc::new(OtxClient::new(
        http_client.clone(),
        cfg.otx_api_key.clone(),
    ));

    // Build enrichment context
    let ctx = Arc::new(EnrichCtx {
        pool: main_pool.clone(),
        redis: redis_conn.clone(),
        vt: vt_client,
        abuse: abuse_client,
        greynoise: greynoise_client,
        ipinfo: ipinfo_client,
        shodan: shodan_client,
        otx: otx_client,
        circuits: circuits.clone(),
        telemetry: telemetry.clone(),
    });

    // j. Start background: cache cleanup
    let cleanup_pool = main_pool.clone();
    let cleanup_hours = cfg.cache_cleanup_hours;
    tokio::spawn(async move {
        cache::cache_cleanup_loop(cleanup_pool, cleanup_hours).await;
    });

    // k. Start background: telemetry flush
    let tele_acc = telemetry.clone();
    let tele_pool = main_pool.clone();
    let tele_secs = cfg.telemetry_flush_secs;
    tokio::spawn(async move {
        telemetry::telemetry_flush_loop(tele_acc, tele_pool, tele_secs).await;
    });

    // l. Connect NATS
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

    // m. Create durable consumer
    let nats_consumer = stream
        .get_or_create_consumer(
            "intel-enricher",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("intel-enricher".to_string()),
                filter_subject: "deepmail.jobs.analysis".to_string(),
                ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                max_deliver: 3,
                ack_wait: std::time::Duration::from_secs(60),
                ..Default::default()
            },
        )
        .await?;

    let consumer_ctx = ctx.clone();
    let consumer_parser = parser_pool.clone();
    let consumer_ingest = ingest_pool.clone();
    let consumer_js = js.clone();
    let consumer_handle = tokio::spawn(async move {
        consumer::run_consumer(nats_consumer, consumer_ctx, consumer_parser, consumer_ingest, consumer_js)
            .await;
    });

    // n. Start gRPC server
    let grpc_addr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    let grpc_service = IntelEnricherService::new(ctx.clone(), circuits.clone());

    tracing::info!(addr = %grpc_addr, "gRPC server starting");

    let grpc_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(IntelEnricherServer::new(grpc_service))
            .serve(grpc_addr)
            .await
    });

    // o. Wait for shutdown
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

    // Final telemetry flush
    if let Err(e) = telemetry.flush_to_db(&main_pool).await {
        tracing::warn!(error = %e, "final telemetry flush failed");
    }

    tracing::info!("deepmail-intel stopped");
    Ok(())
}
