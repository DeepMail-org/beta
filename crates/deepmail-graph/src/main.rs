mod config;
mod consumer;
mod db;
mod error;
mod neo4j;
mod service;
mod sync;

use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::proto::graph::graph_service_server::GraphServiceServer;

use crate::config::GraphConfig;
use crate::neo4j::Neo4jClient;
use crate::service::GraphServiceHandler;
use crate::sync::SyncCtx;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-graph starting");

    let cfg = GraphConfig::load().unwrap_or_else(|e| {
        tracing::warn!("config load failed ({}), using env vars", e);
        GraphConfig {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://deepmail:deepmailpw@localhost:5432/deepmail_graph".to_string()
            }),
            ioc_database_url: std::env::var("IOC_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ioc".to_string()
            }),
            scoring_database_url: std::env::var("SCORING_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://deepmail:deepmailpw@localhost:5432/deepmail_scoring".to_string()
            }),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            grpc_port: std::env::var("GRPC_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50064),
            neo4j_uri: std::env::var("NEO4J_URI")
                .unwrap_or_else(|_| "bolt://localhost:7687".to_string()),
            neo4j_user: std::env::var("NEO4J_USER")
                .unwrap_or_else(|_| "neo4j".to_string()),
            neo4j_password: std::env::var("NEO4J_PASSWORD")
                .unwrap_or_else(|_| "deepmail".to_string()),
            neo4j_database: std::env::var("NEO4J_DATABASE")
                .unwrap_or_else(|_| "neo4j".to_string()),
            sync_concurrency: std::env::var("SYNC_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            max_traversal_depth: std::env::var("MAX_TRAVERSAL_DEPTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
        }
    });

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&cfg.database_url)
            .await?,
    );
    tracing::info!("connected to deepmail_graph DB");

    sqlx::migrate!("../../migrations/deepmail-graph")
        .run(pool.as_ref())
        .await?;
    tracing::info!("migrations applied");

    let ioc_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&cfg.ioc_database_url)?,
    );

    let scoring_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&cfg.scoring_database_url)?,
    );
    tracing::info!("cross-service pools configured (lazy)");

    let neo4j = Neo4jClient::connect(&cfg.neo4j_uri, &cfg.neo4j_user, &cfg.neo4j_password).await?;
    tracing::info!("connected to Neo4j at {}", cfg.neo4j_uri);

    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("connected to NATS");

    let ctx = Arc::new(SyncCtx {
        neo4j,
        pool,
        ioc_pool,
        scoring_pool,
    });

    let consumer_ctx = Arc::clone(&ctx);
    let consumer_nats = nats.clone();
    let concurrency = cfg.sync_concurrency;
    tokio::spawn(async move {
        consumer::run_consumers(consumer_ctx, consumer_nats, concurrency).await;
    });

    let addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%addr, "starting gRPC server");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<GraphServiceServer<GraphServiceHandler>>()
        .await;

    let svc = GraphServiceHandler::new(Arc::clone(&ctx));

    Server::builder()
        .add_service(health_service)
        .add_service(GraphServiceServer::new(svc))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("shutting down");
        })
        .await?;

    Ok(())
}
