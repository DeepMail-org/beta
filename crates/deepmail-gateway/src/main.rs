use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

mod auth_middleware;
mod clients;
mod config;
mod db;
mod error;
mod graphql;
mod middleware;
mod rate_limit;
mod routes;

pub struct GatewayCtx {
    pub pool: Arc<sqlx::PgPool>,
    pub ingest_pool: Arc<sqlx::PgPool>,
    pub redis: redis::aio::ConnectionManager,
    pub http_client: Arc<reqwest::Client>,
    pub auth_client: clients::AuthClient,
    pub scoring_client: clients::ScoringClient,
    pub report_client: clients::ReportClient,
    pub ioc_client: clients::IocClient,
    pub graph_client: clients::GraphClient,
    pub tenant_client: clients::TenantClient,
    pub billing_client: clients::BillingClient,
    pub notify_client: clients::NotifyClient,
    pub graphql_schema: graphql::AppSchema,
    pub config: Arc<config::GatewayConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deepmail_gateway=info,tower_http=info".into()),
        )
        .init();

    info!(service = "deepmail-gateway", "starting");

    // Config — fail fast if required vars missing
    let cfg = config::GatewayConfig::from_env()
        .map_err(|e| anyhow::anyhow!("config error: {e}"))?;
    let cfg = Arc::new(cfg);

    // Database pools
    let pool = Arc::new(
        deepmail_common::db::create_pool(&cfg.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("gateway db pool: {e}"))?,
    );

    let ingest_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&cfg.ingest_database_url)?,
    );

    // Redis
    let redis_client = redis::Client::open(cfg.redis_url.as_str())
        .map_err(|e| anyhow::anyhow!("redis client: {e}"))?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .map_err(|e| anyhow::anyhow!("redis connection: {e}"))?;
    info!("redis connected");

    // HTTP client for proxying
    let http_client = Arc::new(reqwest::Client::new());

    // gRPC clients (all connect_lazy — never block startup)
    let auth_client = clients::AuthClient::new(&cfg.auth_grpc_url)
        .map_err(|e| anyhow::anyhow!("auth client: {e}"))?;
    let scoring_client = clients::ScoringClient::new(&cfg.scoring_grpc_url)
        .map_err(|e| anyhow::anyhow!("scoring client: {e}"))?;
    let report_client = clients::ReportClient::new(&cfg.report_grpc_url)
        .map_err(|e| anyhow::anyhow!("report client: {e}"))?;
    let ioc_client = clients::IocClient::new(&cfg.ioc_grpc_url)
        .map_err(|e| anyhow::anyhow!("ioc client: {e}"))?;
    let graph_client = clients::GraphClient::new(&cfg.graph_grpc_url)
        .map_err(|e| anyhow::anyhow!("graph client: {e}"))?;
    let tenant_client = clients::TenantClient::new(&cfg.tenant_grpc_url)
        .map_err(|e| anyhow::anyhow!("tenant client: {e}"))?;
    let billing_client = clients::BillingClient::new(&cfg.billing_grpc_url)
        .map_err(|e| anyhow::anyhow!("billing client: {e}"))?;
    let notify_client = clients::NotifyClient::new(&cfg.notify_grpc_url)
        .map_err(|e| anyhow::anyhow!("notify client: {e}"))?;

    // GraphQL schema
    let graphql_schema = graphql::build_schema();

    let ctx = Arc::new(GatewayCtx {
        pool,
        ingest_pool,
        redis: redis_conn,
        http_client,
        auth_client,
        scoring_client,
        report_client,
        ioc_client,
        graph_client,
        tenant_client,
        billing_client,
        notify_client,
        graphql_schema,
        config: cfg.clone(),
    });

    // CORS — production-safe: configurable origin
    let cors = if cfg.cors_allow_origin == "*" {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(
                cfg.cors_allow_origin
                    .parse::<HeaderValue>()
                    .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000")),
            )
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(true)
    };

    // Router
    let app = routes::build_router(ctx)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", cfg.http_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("bind error: {e}"))?;

    info!(addr = %addr, "deepmail-gateway listening");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow::anyhow!("server error: {e}"))?;

    info!("deepmail-gateway stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!(error = %e, "ctrl-c handler failed");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut s) => { s.recv().await; }
            Err(e) => {
                tracing::error!(error = %e, "SIGTERM handler failed");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}
