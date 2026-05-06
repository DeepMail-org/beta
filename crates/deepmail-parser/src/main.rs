//! deepmail-parser — Email Parse Worker
//!
//! Pure NATS consumer: no HTTP / gRPC server. Subscribes to
//! `deepmail.jobs.parse`, fetches quarantined email from S3, parses it
//! via RFC 5322, persists structured data, uploads attachments, and
//! publishes to the analysis pipeline.

use std::sync::Arc;

use tokio::signal;
use tracing::info;

mod config;
mod consumer;
mod db;
mod error;
mod parse;
mod s3;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Tracing ──────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "deepmail_parser=info,deepmail_common=info".into()
                }),
        )
        .with_current_span(true)
        .init();

    info!(service = "deepmail-parser", "starting");

    // ── Config ───────────────────────────────────────────────────
    let config = config::Config::load()
        .map_err(|e| anyhow::anyhow!("config error: {e}"))?;
    let config = Arc::new(config);

    // ── Parser database pool ─────────────────────────────────────
    let parser_pool = deepmail_common::db::create_pool(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("parser db pool error: {e}"))?;
    let parser_pool = Arc::new(parser_pool);

    // ── Migrations ───────────────────────────────────────────────
    sqlx::migrate!("../../migrations/deepmail-parser")
        .run(parser_pool.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("migration error: {e}"))?;
    info!("parser database migrations applied");

    // ── Ingest database pool (cross-service progress) ────────────
    let ingest_pool = deepmail_common::db::create_pool(&config.ingest_database_url)
        .await
        .map_err(|e| anyhow::anyhow!("ingest db pool error: {e}"))?;
    let ingest_pool = Arc::new(ingest_pool);

    // ── S3 client ────────────────────────────────────────────────
    let s3_client = Arc::new(s3::create_s3_client(&config).await);
    info!(
        endpoint = %config.s3_endpoint,
        quarantine_bucket = %config.s3_quarantine_bucket,
        attachments_bucket = %config.s3_attachments_bucket,
        "S3 client initialized"
    );

    // ── NATS JetStream ───────────────────────────────────────────
    let nats_js = Arc::new(
        deepmail_common::nats::create_jetstream(&config.nats_url)
            .await
            .map_err(|e| anyhow::anyhow!("NATS connect error: {e}"))?,
    );
    info!(url = %config.nats_url, "NATS JetStream connected");

    // ── Consumer state ───────────────────────────────────────────
    let state = Arc::new(consumer::ConsumerState {
        config: Arc::clone(&config),
        parser_pool,
        ingest_pool,
        s3_client,
        nats_js,
    });

    // ── Run consumer + shutdown signal ───────────────────────────
    let consumer_handle = tokio::spawn({
        let state = Arc::clone(&state);
        async move {
            if let Err(e) = consumer::run_consumer(state).await {
                tracing::error!(error = %e, "consumer loop exited with error");
            }
        }
    });

    // ── Graceful shutdown ────────────────────────────────────────
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

    tokio::select! {
        _ = consumer_handle => {
            info!("consumer task ended");
        }
        _ = shutdown => {
            info!("deepmail-parser shutting down");
        }
    }

    info!("deepmail-parser stopped");
    Ok(())
}
