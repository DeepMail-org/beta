//! deepmail-ingest — File Upload & Quarantine Service
//!
//! HTTP server with a single upload endpoint.
//! No gRPC server — this service is called only by the gateway.

use std::sync::Arc;

use axum::{routing::post, Router};
use tokio::signal;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::info;

mod config;
mod db;
mod error;
mod handler;
mod hash;
mod s3;
mod validate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Tracing ──────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "deepmail_ingest=info,deepmail_common=info".into()
                }),
        )
        .with_current_span(true)
        .init();

    info!(service = "deepmail-ingest", "starting");

    // ── Config ───────────────────────────────────────────────────
    let config = config::Config::load()
        .map_err(|e| anyhow::anyhow!("config error: {e}"))?;
    let config = Arc::new(config);

    // ── Database ─────────────────────────────────────────────────
    let pool = deepmail_common::db::create_pool(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("db pool error: {e}"))?;
    let pool = Arc::new(pool);

    // ── Migrations ───────────────────────────────────────────────
    sqlx::migrate!("../../migrations/deepmail-ingest")
        .run(pool.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("migration error: {e}"))?;
    info!("database migrations applied");

    // ── S3 client ────────────────────────────────────────────────
    let s3_client = Arc::new(s3::create_s3_client(&config).await);
    info!(
        endpoint = %config.s3_endpoint,
        bucket = %config.s3_bucket,
        "S3 client initialized"
    );

    // ── NATS JetStream ───────────────────────────────────────────
    let nats_js = Arc::new(
        deepmail_common::nats::create_jetstream(&config.nats_url)
            .await
            .map_err(|e| anyhow::anyhow!("NATS connect error: {e}"))?,
    );
    info!(url = %config.nats_url, "NATS JetStream connected");

    // ── App state ────────────────────────────────────────────────
    let state = Arc::new(handler::AppState {
        pool,
        config: Arc::clone(&config),
        s3_client,
        nats_js,
    });

    // ── Router ───────────────────────────────────────────────────
    let max_body = config.max_upload_size_mb as usize * 1024 * 1024
        + 1024 * 64; // extra 64 KB for multipart overhead

    let app = Router::new()
        .route("/api/v1/upload", post(handler::upload_handler))
        .with_state(state)
        .layer(RequestBodyLimitLayer::new(max_body))
        .layer(TraceLayer::new_for_http());

    // ── HTTP server ──────────────────────────────────────────────
    let http_addr: std::net::SocketAddr = config
        .http_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid http_addr: {e}"))?;

    info!(addr = %http_addr, "deepmail-ingest HTTP server listening");

    // ── Graceful shutdown future ─────────────────────────────────
    let shutdown = async {
        let ctrl_c = async {
            if let Err(e) = signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to listen for ctrl-c");
                std::future::pending::<()>().await;
            }
        };

        #[cfg(unix)]
        let terminate = async {
            match signal::unix::signal(signal::unix::SignalKind::terminate()) {
                Ok(mut s) => { s.recv().await; }
                Err(e) => {
                    tracing::error!(error = %e, "failed to listen for SIGTERM");
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

        info!("shutdown signal received — draining");
    };

    // axum 0.7 server idiom: bind a TcpListener, then `axum::serve(...)`.
    // The 0.6 `axum::Server::bind(&addr).serve(...)` API was removed.
    let listener = tokio::net::TcpListener::bind(http_addr)
        .await
        .map_err(|e| anyhow::anyhow!("http listener bind error: {e}"))?;

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP server error: {e}"))?;

    info!("deepmail-ingest stopped");
    Ok(())
}
