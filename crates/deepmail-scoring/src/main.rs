mod aggregator;
mod config;
mod consumer;
mod db;
mod error;
mod fetcher;
mod scorer;
mod service;
mod signals;

use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::proto::scoring::threat_scorer_server::ThreatScorerServer;

use crate::config::ScoringConfig;
use crate::fetcher::ServicePools;
use crate::scorer::ScoringCtx;
use crate::service::ScoringService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-scoring starting");

    let cfg = ScoringConfig::load().unwrap_or_else(|e| {
        tracing::warn!("config load failed ({}), using env vars", e);
        ScoringConfig {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_scoring".to_string()),
            ingest_database_url: std::env::var("INGEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ingest".to_string()),
            header_database_url: std::env::var("HEADER_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_header".to_string()),
            dkim_database_url: std::env::var("DKIM_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_dkim".to_string()),
            geo_database_url: std::env::var("GEO_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_geo".to_string()),
            ip_database_url: std::env::var("IP_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ip".to_string()),
            intel_database_url: std::env::var("INTEL_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_intel".to_string()),
            ioc_database_url: std::env::var("IOC_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ioc".to_string()),
            homograph_database_url: std::env::var("HOMOGRAPH_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_homograph".to_string()),
            body_database_url: std::env::var("BODY_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_body".to_string()),
            sandbox_url_database_url: std::env::var("SANDBOX_URL_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_url".to_string()),
            sandbox_file_database_url: std::env::var("SANDBOX_FILE_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_file".to_string()),
            sandbox_dynamic_database_url: std::env::var("SANDBOX_DYNAMIC_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_dynamic".to_string()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            grpc_port: std::env::var("GRPC_PORT")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(50063),
            concurrency: std::env::var("CONCURRENCY")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(10),
            score_final_min_signals: std::env::var("SCORE_FINAL_MIN_SIGNALS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(8),
            score_timeout_minutes: std::env::var("SCORE_TIMEOUT_MINUTES")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(5),
        }
    });

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(15)
            .connect(&cfg.database_url)
            .await?,
    );
    tracing::info!("connected to deepmail_scoring DB");

    sqlx::migrate!("../../migrations/deepmail-scoring")
        .run(pool.as_ref())
        .await?;
    tracing::info!("migrations applied");

    let ingest_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&cfg.ingest_database_url)?,
    );

    let service_pools = Arc::new(ServicePools {
        header: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.header_database_url)?),
        dkim: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.dkim_database_url)?),
        geo: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.geo_database_url)?),
        ip: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.ip_database_url)?),
        intel: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.intel_database_url)?),
        ioc: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.ioc_database_url)?),
        homograph: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.homograph_database_url)?),
        body: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.body_database_url)?),
        sandbox_url: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.sandbox_url_database_url)?),
        sandbox_file: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.sandbox_file_database_url)?),
        sandbox_dynamic: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.sandbox_dynamic_database_url)?),
    });
    tracing::info!("service pools configured (lazy)");

    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("connected to NATS");

    let ctx = Arc::new(ScoringCtx {
        pool,
        service_pools,
        ingest_pool,
        nats,
        final_min_signals: cfg.score_final_min_signals,
    });

    let consumer_ctx = Arc::clone(&ctx);
    let concurrency = cfg.concurrency;
    let timeout_minutes = cfg.score_timeout_minutes;
    tokio::spawn(async move {
        consumer::run_consumers(consumer_ctx, concurrency, timeout_minutes).await;
    });

    let addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%addr, "starting gRPC server");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ThreatScorerServer<ScoringService>>()
        .await;

    let svc = ScoringService::new(Arc::clone(&ctx));

    Server::builder()
        .add_service(health_service)
        .add_service(ThreatScorerServer::new(svc))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("shutting down");
        })
        .await?;

    Ok(())
}
