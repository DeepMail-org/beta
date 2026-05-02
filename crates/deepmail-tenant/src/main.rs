//! deepmail-tenant — Tenant & Organization Service
//!
//! Runs the HTTP server for the Razorpay webhook receiver on
//! `config.http_addr`. The gRPC server (planned on `config.grpc_addr`)
//! will be wired in once tenant.proto lands in deepmail-common.

use std::sync::Arc;

use axum::{routing::post, Router};
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod db;
mod error;
mod service;
mod webhook;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Tracing ──────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "deepmail_tenant=info,deepmail_common=info".into()
                }),
        )
        .with_current_span(true)
        .init();

    info!(service = "deepmail-tenant", "starting");

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
    sqlx::migrate!("../../migrations/deepmail-tenant")
        .run(pool.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("migration error: {e}"))?;
    info!("database migrations applied");

    // ── Webhook HTTP server ───────────────────────────────────────
    let webhook_state = Arc::new(webhook::WebhookState {
        pool: Arc::clone(&pool),
        config: Arc::clone(&config),
    });

    let http_app = Router::new()
        .route(
            "/webhooks/razorpay",
            post(webhook::handle_razorpay_webhook),
        )
        .with_state(webhook_state)
        .layer(TraceLayer::new_for_http());

    let http_addr: std::net::SocketAddr = config
        .http_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid http_addr: {e}"))?;

    info!(addr = %http_addr, "deepmail-tenant HTTP webhook server listening");

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

    axum::serve(listener, http_app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|e| anyhow::anyhow!("http server error: {e}"))?;

    info!("deepmail-tenant stopped");
    Ok(())
}
